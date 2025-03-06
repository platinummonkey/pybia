use file_watcher::{
    service::{
        detector::ServiceDetector,
        models::{ServiceConfig, ServiceDetectionRules},
    },
    dependency::DependencyGraph,
};
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_project() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    
    // Create service1
    std::fs::create_dir_all(temp.path().join("service1/src")).unwrap();
    std::fs::write(
        temp.path().join("service1/setup.py"),
        r#"setup(name='service1')"#,
    ).unwrap();
    std::fs::write(
        temp.path().join("service1/src/__init__.py"),
        "",
    ).unwrap();
    std::fs::write(
        temp.path().join("service1/requirements.txt"),
        "requests==2.26.0",
    ).unwrap();
    
    // Create service2
    std::fs::create_dir_all(temp.path().join("service2")).unwrap();
    std::fs::write(
        temp.path().join("service2/pyproject.toml"),
        r#"
        [project]
        name = "service2"
        dependencies = ["service1"]
        "#,
    ).unwrap();
    
    temp
}

#[test]
fn test_full_service_detection_and_dependencies() {
    let temp = setup_test_project();
    
    // Configure services
    let config = ServiceConfig {
        name: "service3".to_string(),
        path: temp.path().join("service3"),
        include_paths: vec![],
        exclude_paths: vec![],
        detection: ServiceDetectionRules::default(),
    };
    
    // Detect services
    let detector = ServiceDetector::new(vec![config]);
    let services = detector.detect_services(temp.path()).unwrap();
    
    assert_eq!(services.len(), 2);
    assert!(services.contains_key("service1"));
    assert!(services.contains_key("service2"));
    
    // Build dependency graph
    let mut graph = DependencyGraph::new();
    graph.build_from_directory(temp.path(), services).unwrap();
    
    // Test dependencies
    let affected = graph.get_affected_services(
        &temp.path().join("service1/requirements.txt")
    );
    
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].0, "service1");
} 