use super::models::{DetectedService, ServiceConfig, ServiceDetectionType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ServiceDetector {
    configs: Vec<ServiceConfig>,
}

impl ServiceDetector {
    pub fn new(configs: Vec<ServiceConfig>) -> Self {
        Self { configs }
    }

    pub fn detect_services(&self, root_path: &Path) -> std::io::Result<HashMap<String, DetectedService>> {
        let mut services = HashMap::new();

        // First, add configured services
        for config in &self.configs {
            let service = DetectedService {
                name: config.name.clone(),
                root_path: config.path.clone(),
                package_root: self.find_package_root(&config.path)?,
                detection_type: ServiceDetectionType::ConfigurationDefined,
            };
            services.insert(service.name.clone(), service);
        }

        // Then detect additional services
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip if path is in an existing service
            if self.is_path_in_existing_service(path, &services) {
                continue;
            }

            if let Some(service) = self.detect_service_at_path(path)? {
                if !services.contains_key(&service.name) {
                    services.insert(service.name.clone(), service);
                }
            }
        }

        Ok(services)
    }

    fn detect_service_at_path(&self, path: &Path) -> std::io::Result<Option<DetectedService>> {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return Ok(None),
        };

        match file_name {
            "setup.py" => {
                let root_path = path.parent().unwrap_or(path).to_path_buf();
                let name = self.extract_package_name_from_setup_py(path)?
                    .unwrap_or_else(|| root_path.file_name().unwrap().to_string_lossy().into_owned());
                
                Ok(Some(DetectedService {
                    name,
                    root_path: root_path.clone(),
                    package_root: self.find_package_root(&root_path)?,
                    detection_type: ServiceDetectionType::SetupPy,
                }))
            }
            "pyproject.toml" => {
                let root_path = path.parent().unwrap_or(path).to_path_buf();
                let name = self.extract_package_name_from_pyproject(path)?
                    .unwrap_or_else(|| root_path.file_name().unwrap().to_string_lossy().into_owned());

                Ok(Some(DetectedService {
                    name,
                    root_path: root_path.clone(),
                    package_root: self.find_package_root(&root_path)?,
                    detection_type: ServiceDetectionType::PyprojectToml,
                }))
            }
            _ => Ok(None),
        }
    }

    fn extract_package_name_from_setup_py(&self, path: &Path) -> std::io::Result<Option<String>> {
        let content = std::fs::read_to_string(path)?;
        
        for line in content.lines() {
            let line = line.trim();
            if line.contains("name=") || line.contains("name =") {
                // Extract the name parameter
                let name_part = if let Some(idx) = line.find("name=") {
                    &line[idx + 5..]
                } else if let Some(idx) = line.find("name =") {
                    &line[idx + 6..]
                } else {
                    continue;
                };
                
                // Handle different formats: name='value', name="value", name=value
                let cleaned_name = name_part
                    .trim_start()
                    .trim_start_matches('(')
                    .trim_start_matches('\'')
                    .trim_start_matches('"')
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches('\'')
                    .trim_end_matches('"')
                    .trim_end_matches(')')
                    .trim_end_matches(',')
                    .to_string();
                    
                if !cleaned_name.is_empty() {
                    return Ok(Some(cleaned_name));
                }
            }
        }
        
        Ok(None)
    }

    fn extract_package_name_from_pyproject(&self, path: &Path) -> std::io::Result<Option<String>> {
        let content = std::fs::read_to_string(path)?;
        let toml: toml::Value = toml::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;

        Ok(toml.get("project")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(String::from))
    }

    fn find_package_root(&self, dir: &Path) -> std::io::Result<PathBuf> {
        // Look for __init__.py to determine package root
        for entry in WalkDir::new(dir)
            .max_depth(3)  // Limit depth to avoid searching too deep
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "__init__.py" {
                return Ok(entry.path().parent().unwrap_or(dir).to_path_buf());
            }
        }
        Ok(dir.to_path_buf())
    }

    fn is_path_in_existing_service(&self, path: &Path, services: &HashMap<String, DetectedService>) -> bool {
        // Don't skip nested services
        false
    }
} 