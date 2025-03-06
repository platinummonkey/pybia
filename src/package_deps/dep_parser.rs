use super::models::{DependencyFile, DependencyFileKind, PackageRequirement};
use std::path::Path;
use std::fs;
use regex::Regex;
use lazy_static::lazy_static;
use semver::VersionReq;
use toml;

lazy_static! {
    static ref REQUIREMENT_REGEX: Regex = Regex::new(concat!(
        r"^(?P<name>[A-Za-z0-9][A-Za-z0-9._-]*[A-Za-z0-9])",
        r"(?:\[(?P<extras>[^\]]+)\])?",
        r"(?P<constraint>(?:<=?|>=?|==|!=|~=|===)[^;,\s]*)",
        r"(?:\s*;\s*(?P<marker>.*))?$"
    )).unwrap();
}

pub struct DependencyParser;

impl DependencyParser {
    pub fn parse_file(path: &Path) -> std::io::Result<DependencyFile> {
        let content = fs::read_to_string(path)?;
        let kind = Self::determine_file_kind(path);
        
        let dependencies = match kind {
            DependencyFileKind::RequirementsTxt | DependencyFileKind::RequirementsIn => {
                Self::parse_requirements(&content)
            }
            DependencyFileKind::SetupPy => Self::parse_setup_py(&content),
            DependencyFileKind::PyprojectToml | DependencyFileKind::Poetry => {
                Self::parse_pyproject_toml(&content)
            }
            DependencyFileKind::Pipfile => Self::parse_pipfile(&content),
        };

        Ok(DependencyFile {
            path: path.to_path_buf(),
            kind,
            dependencies,
        })
    }

    fn determine_file_kind(path: &Path) -> DependencyFileKind {
        match path.file_name().and_then(|n| n.to_str()) {
            Some("requirements.txt") => DependencyFileKind::RequirementsTxt,
            Some("requirements.in") => DependencyFileKind::RequirementsIn,
            Some("setup.py") => DependencyFileKind::SetupPy,
            Some("pyproject.toml") => {
                // Check if it's Poetry by looking for [tool.poetry] section
                if let Ok(content) = fs::read_to_string(path) {
                    if content.contains("[tool.poetry]") {
                        DependencyFileKind::Poetry
                    } else {
                        DependencyFileKind::PyprojectToml
                    }
                } else {
                    DependencyFileKind::PyprojectToml
                }
            }
            Some("Pipfile") => DependencyFileKind::Pipfile,
            _ => DependencyFileKind::RequirementsTxt,
        }
    }

    fn parse_requirements(content: &str) -> Vec<PackageRequirement> {
        content.lines()
            .filter(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with('#') && !line.starts_with('-')
            })
            .filter_map(Self::parse_requirement_line)
            .collect()
    }

    fn parse_requirement_line(line: &str) -> Option<PackageRequirement> {
        let captures = REQUIREMENT_REGEX.captures(line)?;
        
        Some(PackageRequirement {
            name: captures.name("name")?.as_str().to_string(),
            version_req: captures.name("constraint")
                .and_then(|c| VersionReq::parse(c.as_str()).ok()),
            extras: captures.name("extras")
                .map(|e| e.as_str().split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            marker: captures.name("marker")
                .map(|m| m.as_str().to_string()),
        })
    }

    fn parse_setup_py(content: &str) -> Vec<PackageRequirement> {
        // Basic implementation - extract install_requires
        let mut requirements = Vec::new();
        if let Some(start) = content.find("install_requires=[") {
            if let Some(end) = content[start..].find(']') {
                let deps = &content[start + "install_requires=[".len()..start + end];
                for dep in deps.split(',') {
                    if let Some(req) = Self::parse_requirement_line(dep.trim().trim_matches('\'').trim_matches('"')) {
                        requirements.push(req);
                    }
                }
            }
        }
        requirements
    }

    fn parse_pyproject_toml(content: &str) -> Vec<PackageRequirement> {
        let mut requirements = Vec::new();
        
        if let Ok(toml) = content.parse::<toml::Value>() {
            // Check project dependencies array format
            if let Some(deps) = toml.get("project")
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_array()) {
                for dep in deps {
                    if let Some(dep_str) = dep.as_str() {
                        if let Some(req) = Self::parse_requirement_line(dep_str) {
                            requirements.push(req);
                        }
                    }
                }
            }
            
            // Check project dependencies table format
            if let Some(deps) = toml.get("project")
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_table()) {
                for (name, constraint) in deps {
                    if let Some(version) = constraint.as_str() {
                        if let Some(req) = Self::parse_requirement_line(&format!("{}=={}", name, version)) {
                            requirements.push(req);
                        }
                    }
                }
            }
        }
        
        requirements
    }

    fn parse_pipfile(content: &str) -> Vec<PackageRequirement> {
        let mut requirements = Vec::new();
        
        if let Ok(toml) = content.parse::<toml::Value>() {
            // Check both packages and dev-packages sections
            for section in ["packages", "dev-packages"] {
                if let Some(deps) = toml.get(section).and_then(|p| p.as_table()) {
                    for (name, constraint) in deps {
                        let version_req = match constraint {
                            toml::Value::String(v) => Some(v.as_str()),
                            toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str()),
                            _ => None,
                        };

                        if let Some(version) = version_req {
                            if let Some(req) = Self::parse_requirement_line(&format!("{}=={}", name, version)) {
                                requirements.push(req);
                            }
                        } else {
                            requirements.push(PackageRequirement {
                                name: name.clone(),
                                version_req: None,
                                extras: Vec::new(),
                                marker: None,
                            });
                        }
                    }
                }
            }
        }
        
        requirements
    }
} 