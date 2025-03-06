use std::path::Path;

pub fn is_python_file(path: &Path) -> bool {
    path.extension()
        .map_or(false, |ext| ext == "py")
}

pub fn normalize_path(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
} 