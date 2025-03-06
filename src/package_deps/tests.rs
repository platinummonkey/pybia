use super::*;
use tempfile::TempDir;

fn setup_package_test() -> (TempDir, PackageDependencyManager) {
    let temp = tempfile::tempdir().unwrap();
    let manager = PackageDependencyManager::new();
    (temp, manager)
}

#[test]
fn test_requirements_txt_parsing() {
    let (temp, mut manager) = setup_package_test();
    
    std::fs::write(
        temp.path().join("requirements.txt"),
        "requests==2.26.0\nflask>=1.0.0\npytest\n",
    ).unwrap();
    
    std::fs::write(
        temp.path().join("app.py"),
        "import requests\nfrom flask import Flask\n",
    ).unwrap();

    manager.scan_directory(temp.path()).unwrap();
    
    let affected = manager.get_affected_by_dependency_change(
        &temp.path().join("requirements.txt")
    );
    
    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&temp.path().join("app.py")));
}

#[test]
fn test_pyproject_toml_parsing() {
    let (temp, mut manager) = setup_package_test();
    
    std::fs::write(
        temp.path().join("pyproject.toml"),
        r#"
        [project]
        dependencies = [
            "requests>=2.26.0",
            "flask>=1.0.0"
        ]
        "#,
    ).unwrap();
    
    std::fs::write(
        temp.path().join("app.py"),
        r#"
        import requests
        from flask import Flask
        "#,
    ).unwrap();

    manager.scan_directory(temp.path()).unwrap();
    
    let affected = manager.get_affected_by_dependency_change(
        &temp.path().join("pyproject.toml")
    );
    
    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&temp.path().join("app.py")));
}

#[test]
fn test_import_parsing() {
    let (temp, mut manager) = setup_package_test();
    
    std::fs::write(
        temp.path().join("test.py"),
        r#"
        import requests
        from flask import Flask
        from .utils import helper
        import os, sys
        from typing import List, Optional
        "#,
    ).unwrap();

    manager.scan_directory(temp.path()).unwrap();
    
    let imports = manager.import_cache.get(&temp.path().join("test.py")).unwrap();
    
    assert_eq!(imports.len(), 6);
    assert!(imports.iter().any(|i| i.package_name == "requests"));
    assert!(imports.iter().any(|i| i.package_name == "flask"));
    assert!(imports.iter().any(|i| i.package_name == "os"));
    assert!(imports.iter().any(|i| i.package_name == "sys"));
    assert!(imports.iter().any(|i| i.package_name == "typing"));
} 