use super::*;
use crate::service::models::{DetectedService, ServiceDetectionType};
use std::collections::HashMap;
use tempfile::TempDir;

fn setup_test_env() -> (TempDir, DependencyGraph) {
    let temp = tempfile::tempdir().unwrap();
    let mut services = HashMap::new();
    
    // Create test services
    services.insert(
        "service1".to_string(),
        DetectedService {
            name: "service1".to_string(),
            root_path: temp.path().join("service1"),
            package_root: temp.path().join("service1/src"),
            detection_type: ServiceDetectionType::SetupPy,
        },
    );
    
    services.insert(
        "service2".to_string(),
        DetectedService {
            name: "service2".to_string(),
            root_path: temp.path().join("service2"),
            package_root: temp.path().join("service2/src"),
            detection_type: ServiceDetectionType::SetupPy,
        },
    );

    let mut graph = DependencyGraph::new();
    graph.build_from_directory(temp.path(), services).unwrap();
    
    (temp, graph)
}

#[test]
fn test_service_dependencies() {
    let (temp, mut graph) = setup_test_env();
    
    // Create test files
    std::fs::create_dir_all(temp.path().join("service1/src")).unwrap();
    std::fs::create_dir_all(temp.path().join("service2/src")).unwrap();
    
    std::fs::write(
        temp.path().join("service1/src/main.py"),
        "from service2.utils import helper",
    ).unwrap();
    
    std::fs::write(
        temp.path().join("service2/src/utils.py"),
        "def helper(): pass",
    ).unwrap();

    // Add dependency
    graph.add_dependency(
        temp.path().join("service1/src/main.py"),
        temp.path().join("service2/src/utils.py"),
    );

    // Test affected services
    let affected = graph.get_affected_services(
        &temp.path().join("service2/src/utils.py")
    );
    
    assert_eq!(affected.len(), 2);
    assert!(affected.iter().any(|(name, _)| *name == "service1"));
    assert!(affected.iter().any(|(name, _)| *name == "service2"));
}

#[test]
fn test_package_dependencies() {
    let (temp, mut graph) = setup_test_env();
    
    // Create requirements.txt in service1
    std::fs::create_dir_all(temp.path().join("service1")).unwrap();
    std::fs::write(
        temp.path().join("service1/requirements.txt"),
        "requests==2.26.0",
    ).unwrap();
    
    // Create Python file using requests in service1
    std::fs::create_dir_all(temp.path().join("service1/src")).unwrap();
    std::fs::write(
        temp.path().join("service1/src/api.py"),
        "import requests",
    ).unwrap();

    // Rebuild graph with new files
    graph.build_from_directory(temp.path(), graph.services.clone()).unwrap();

    // Test affected services when requirements.txt changes
    let affected = graph.get_affected_services(
        &temp.path().join("service1/requirements.txt")
    );
    
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].0, "service1");
} 