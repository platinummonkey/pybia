use semver::{Version, VersionReq};
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PackageRequirement {
    pub name: String,
    pub version_req: Option<VersionReq>,
    pub extras: Vec<String>,
    pub marker: Option<String>,
}

#[derive(Debug)]
pub struct DependencyFile {
    pub path: PathBuf,
    pub kind: DependencyFileKind,
    pub dependencies: Vec<PackageRequirement>,
}

#[derive(Debug, PartialEq)]
pub enum DependencyFileKind {
    RequirementsTxt,
    RequirementsIn,
    SetupPy,
    PyprojectToml,
    Poetry,
    Pipfile,
}

#[derive(Debug)]
pub struct ImportInfo {
    pub package_name: String,
    pub module_path: Vec<String>,
    pub is_from_import: bool,
    pub imported_names: Vec<String>,
} 