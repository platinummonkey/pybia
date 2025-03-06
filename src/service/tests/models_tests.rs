use super::super::models::*;

#[test]
fn test_service_detection_rules_default() {
    let rules = ServiceDetectionRules::default();
    assert!(rules.detect_setup_py);
    assert!(rules.detect_pyproject);
    assert!(rules.indicator_files.is_empty());
} 