use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use crate::package_deps::PackageDependencyManager;
use crate::service::models::DetectedService;

#[derive(Debug)]
pub struct DependencyGraph {
    // Map from file to its direct dependencies
    deps: HashMap<PathBuf, HashSet<PathBuf>>,
    // Reverse map from file to files that depend on it
    reverse_deps: HashMap<PathBuf, HashSet<PathBuf>>,
    // Package dependencies
    package_deps: PackageDependencyManager,
    // Services
    services: HashMap<String, DetectedService>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            deps: HashMap::new(),
            reverse_deps: HashMap::new(),
            package_deps: PackageDependencyManager::new(),
            services: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        // Add direct dependency
        self.deps.entry(from.clone())
            .or_default()
            .insert(to.clone());

        // Add reverse dependency
        self.reverse_deps.entry(to)
            .or_default()
            .insert(from);
    }

    pub fn get_affected_files(&self, changed_file: &Path) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();
        
        // Check if it's a dependency file
        if self.package_deps.is_dependency_file(changed_file) {
            affected.extend(self.package_deps.get_affected_by_dependency_change(changed_file));
        }
        
        // Get regular file dependencies
        self.collect_affected_files(changed_file, &mut affected);
        affected
    }

    fn collect_affected_files(&self, file: &Path, affected: &mut HashSet<PathBuf>) {
        if affected.insert(file.to_path_buf()) {
            if let Some(dependents) = self.reverse_deps.get(file) {
                for dependent in dependents {
                    self.collect_affected_files(dependent, affected);
                }
            }
        }
    }

    pub fn get_affected_services(&self, changed_file: &Path) -> Vec<(&str, &Path)> {
        let affected_files = self.get_affected_files(changed_file);
        let mut affected_services = HashMap::new();

        for file in affected_files {
            for (name, service) in &self.services {
                if file.starts_with(&service.root_path) {
                    affected_services.insert(name.as_str(), service.root_path.as_path());
                    break;
                }
            }
        }

        affected_services.into_iter().collect()
    }

    pub fn build_from_directory(&mut self, dir: &Path, services: HashMap<String, DetectedService>) -> std::io::Result<()> {
        self.services = services;
        self.package_deps.scan_directory(dir)?;

        let walker = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(path) {
                    // Scan for regular Python imports
                    for line in content.lines() {
                        if line.starts_with("from ") || line.starts_with("import ") {
                            let imported = line.split_whitespace().nth(1).unwrap();
                            let import_path = dir.join(imported.replace(".", "/") + ".py");
                            if import_path.exists() {
                                self.add_dependency(path.to_path_buf(), import_path);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests; 