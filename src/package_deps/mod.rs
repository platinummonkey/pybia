pub mod models;
pub mod import_parser;
pub mod dep_parser;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use models::{DependencyFile, ImportInfo};
use import_parser::ImportParser;
use dep_parser::DependencyParser;

#[derive(Debug)]
pub struct PackageDependencyManager {
    // Map of package name to files that import it
    package_usages: HashMap<String, HashSet<PathBuf>>,
    // Map of dependency file to its parsed contents
    dependency_files: HashMap<PathBuf, DependencyFile>,
    // Cache of parsed imports by file
    import_cache: HashMap<PathBuf, Vec<ImportInfo>>,
}

impl Default for PackageDependencyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageDependencyManager {
    pub fn new() -> Self {
        Self {
            package_usages: HashMap::new(),
            dependency_files: HashMap::new(),
            import_cache: HashMap::new(),
        }
    }

    pub fn scan_directory(&mut self, dir: &Path) -> std::io::Result<()> {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if self.is_dependency_file(path) {
                if let Ok(dep_file) = DependencyParser::parse_file(path) {
                    self.dependency_files.insert(path.to_path_buf(), dep_file);
                }
            } else if self.is_python_file(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    self.scan_python_file(path, &content)?;
                }
            }
        }
        Ok(())
    }

    pub fn scan_python_file(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        let mut parser = ImportParser::new(content.to_string());
        let imports = parser.parse_imports();
        
        for import_info in &imports {
            self.package_usages
                .entry(import_info.package_name.clone())
                .or_default()
                .insert(path.to_path_buf());
        }
        
        self.import_cache.insert(path.to_path_buf(), imports);
        Ok(())
    }

    pub fn get_affected_by_dependency_change(&self, changed_file: &Path) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();
        
        if let Some(dep_file) = self.dependency_files.get(changed_file) {
            for req in &dep_file.dependencies {
                if let Some(files) = self.package_usages.get(&req.name) {
                    affected.extend(files.iter().cloned());
                }
            }
        }
        
        affected
    }

    pub fn is_dependency_file(&self, path: &Path) -> bool {
        matches!(path.file_name().and_then(|n| n.to_str()),
            Some("requirements.txt") |
            Some("requirements.in") |
            Some("setup.py") |
            Some("pyproject.toml") |
            Some("Pipfile"))
    }

    pub fn is_python_file(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "py")
    }
}

#[cfg(test)]
mod tests; 