use super::super::{
    detector::ServiceDetector,
    models::{ServiceConfig, ServiceDetectionType, ServiceDetectionRules},
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn create_test_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn create_file(root: &Path, path: &str, content: &str) {
    let full_path = root.join(path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full_path, content).unwrap();
}

#[test]
fn test_detect_setup_py_service() {
    let temp = create_test_dir();
    create_file(
        temp.path(),
        "service1/setup.py",
        r#"from setuptools import setup
        setup(
            name='test-service',
            version='0.1.0',
        )
        "#,
    );
    create_file(temp.path(), "service1/src/__init__.py", "");

    let detector = ServiceDetector::new(vec![]);
    let services = detector.detect_services(temp.path()).unwrap();

    assert_eq!(services.len(), 1);
    let service = services.get("test-service").unwrap();
    assert_eq!(service.detection_type, ServiceDetectionType::SetupPy);
    assert!(service.root_path.ends_with("service1"));
}

#[test]
fn test_detect_pyproject_toml_service() {
    let temp = create_test_dir();
    create_file(
        temp.path(),
        "service2/pyproject.toml",
        r#"
        [project]
        name = "api-service"
        version = "0.1.0"
        "#,
    );

    let detector = ServiceDetector::new(vec![]);
    let services = detector.detect_services(temp.path()).unwrap();

    assert_eq!(services.len(), 1);
    let service = services.get("api-service").unwrap();
    assert_eq!(service.detection_type, ServiceDetectionType::PyprojectToml);
}

#[test]
fn test_configured_service() {
    let temp = create_test_dir();
    create_file(temp.path(), "custom-service/src/__init__.py", "");

    let config = ServiceConfig {
        name: "custom-service".to_string(),
        path: temp.path().join("custom-service"),
        include_paths: vec![],
        exclude_paths: vec![],
        detection: ServiceDetectionRules::default(),
    };

    let detector = ServiceDetector::new(vec![config]);
    let services = detector.detect_services(temp.path()).unwrap();

    assert_eq!(services.len(), 1);
    let service = services.get("custom-service").unwrap();
    assert_eq!(service.detection_type, ServiceDetectionType::ConfigurationDefined);
}

#[test]
fn test_nested_services() {
    let temp = create_test_dir();
    create_file(
        temp.path(),
        "parent/setup.py",
        r#"from setuptools import setup
        setup(name='parent-service')
        "#,
    );
    create_file(
        temp.path(),
        "parent/child/setup.py",
        r#"from setuptools import setup
        setup(name='child-service')
        "#,
    );
    create_file(temp.path(), "parent/src/__init__.py", "");
    create_file(temp.path(), "parent/child/src/__init__.py", "");

    let detector = ServiceDetector::new(vec![]);
    let services = detector.detect_services(temp.path()).unwrap();

    assert_eq!(services.len(), 2);
    assert!(services.contains_key("parent-service"));
    assert!(services.contains_key("child-service"));
} 