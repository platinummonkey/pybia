use std::path::{PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Name of the service
    pub name: String,
    /// Root path of the service
    pub path: PathBuf,
    /// Optional list of additional paths to include
    #[serde(default)]
    pub include_paths: Vec<PathBuf>,
    /// Optional list of paths to exclude
    #[serde(default)]
    pub exclude_paths: Vec<PathBuf>,
    /// Optional service detection rules
    #[serde(default)]
    pub detection: ServiceDetectionRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetectionRules {
    /// Look for setup.py files
    #[serde(default = "default_true")]
    pub detect_setup_py: bool,
    /// Look for pyproject.toml files
    #[serde(default = "default_true")]
    pub detect_pyproject: bool,
    /// Additional files that indicate a service root
    #[serde(default)]
    pub indicator_files: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ServiceDetectionRules {
    fn default() -> Self {
        Self {
            detect_setup_py: true,
            detect_pyproject: true,
            indicator_files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedService {
    pub name: String,
    pub root_path: PathBuf,
    pub package_root: PathBuf,
    pub detection_type: ServiceDetectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceDetectionType {
    SetupPy,
    PyprojectToml,
    ConfigurationDefined,
    IndicatorFile(String),
} 