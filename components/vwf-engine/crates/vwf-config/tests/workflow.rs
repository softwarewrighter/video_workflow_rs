//! Workflow configuration tests.

use vwf_config::WorkflowConfig;

#[test]
fn parses_minimal_workflow() {
    let yaml = r#"
version: 1
name: test
steps:
  - id: d
    kind: ensure_dirs
    dirs: ["work"]
"#;
    let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
    assert_eq!(cfg.name, "test");
    assert_eq!(cfg.steps.len(), 1);
    assert_eq!(cfg.steps[0].id, "d");
}

#[test]
fn unknown_step_kind_errors() {
    let yaml = r#"
version: 1
name: test
steps:
  - id: bad_step
    kind: unknown_kind
    foo: bar
"#;
    let err = WorkflowConfig::from_yaml(yaml).unwrap_err().to_string();
    assert!(err.contains("unknown_kind") || err.contains("unknown variant"));
}

#[test]
fn duplicate_step_id_errors() {
    let yaml = r#"
version: 1
name: test
steps:
  - id: same
    kind: ensure_dirs
    dirs: ["a"]
  - id: same
    kind: ensure_dirs
    dirs: ["b"]
"#;
    let err = WorkflowConfig::from_yaml(yaml).unwrap_err().to_string();
    assert!(err.contains("Duplicate step id") && err.contains("same"));
}

#[test]
fn empty_step_id_errors() {
    let yaml = r#"
version: 1
name: test
steps:
  - id: ""
    kind: ensure_dirs
    dirs: ["a"]
"#;
    let err = WorkflowConfig::from_yaml(yaml).unwrap_err().to_string();
    assert!(err.contains("empty"));
}

#[test]
fn vars_substitution_in_workflow() {
    let yaml = r#"
version: 1
name: test
vars:
  project: demo
  output_dir: work
steps:
  - id: d
    kind: ensure_dirs
    dirs: ["{{output_dir}}"]
"#;
    let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
    assert_eq!(cfg.vars.get("project"), Some(&"demo".to_string()));
    assert_eq!(cfg.vars.get("output_dir"), Some(&"work".to_string()));
}
