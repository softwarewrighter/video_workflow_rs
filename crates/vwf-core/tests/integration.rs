//! Integration tests for vwf-core workflow engine.

use std::collections::BTreeMap;
use tempfile::TempDir;
use vwf_core::{DryRunRuntime, FsRuntime, MockLlmClient, Runner, Runtime, StepStatus, WorkflowConfig};

mod workflow_execution {
    use super::*;

    #[test]
    fn produces_manifest_and_artifacts() {
        let yaml = r#"
version: 1
name: integration_test
vars:
  greeting: Hello
steps:
  - id: create_dirs
    kind: ensure_dirs
    dirs: ["output", "work"]
  - id: write_greeting
    kind: write_file
    path: "output/greeting.txt"
    content: "{{greeting}}, World!"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let report = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert_eq!(report.workflow_name, "integration_test");
        assert!(report.steps.iter().all(|s| matches!(s.status, StepStatus::Ok)));
        assert_eq!(rt.read_text("output/greeting.txt").unwrap(), "Hello, World!");
    }

    #[test]
    fn var_override_takes_precedence() {
        let yaml = "version: 1\nname: test\nvars:\n  name: default\nsteps:\n  - id: w\n    kind: write_file\n    path: out.txt\n    content: \"Hello, {{name}}!\"";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let mut overrides = BTreeMap::new();
        overrides.insert("name".into(), "Override".into());
        Runner::run(&mut rt, &cfg, overrides).unwrap();
        assert_eq!(rt.read_text("out.txt").unwrap(), "Hello, Override!");
    }
}

mod dry_run {
    use super::*;

    #[test]
    fn does_not_mutate_filesystem() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: d\n    kind: ensure_dirs\n    dirs: [\"x\"]\n  - id: w\n    kind: write_file\n    path: x/f.txt\n    content: test";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = DryRunRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let report = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert_eq!(report.steps.len(), 2);
        assert!(!tmp.path().join("x").exists());
        assert!(rt.planned_dirs.contains(&"x".to_string()));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn missing_var_fails_with_context() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: w\n    kind: write_file\n    path: out.txt\n    content: \"{{undefined}}\"";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("undefined"));
    }

    #[test]
    fn step_failure_includes_step_id() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: bad_step\n    kind: split_sections\n    input_path: missing.txt\n    outputs: []";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("bad_step"));
    }
}

mod llm_integration {
    use super::*;

    #[test]
    fn generates_with_mock_response() {
        let yaml = r#"
version: 1
name: test
steps:
  - id: prompt
    kind: write_file
    path: prompt.txt
    content: "Generate"
  - id: llm
    kind: llm_generate
    system: "sys"
    user_prompt_path: prompt.txt
    output_path: out.txt
    provider: mock
    mock_response: "MOCK OUTPUT"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert!(rt.read_text("out.txt").unwrap().contains("MOCK"));
    }
}

mod command_execution {
    use super::*;

    #[test]
    fn without_allowlist_fails() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: cmd\n    kind: run_command\n    program: echo\n    args: [hi]";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        rt.command_allowlist.insert("other".into());
        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("not allowed") || err.to_string().contains("allowlist"));
    }

    #[test]
    fn with_allowlist_succeeds() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: cmd\n    kind: run_command\n    program: echo\n    args: [hello]\n    capture_path: out.txt";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        rt.command_allowlist.insert("echo".into());
        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert!(rt.read_text("out.txt").unwrap().contains("hello"));
    }

    #[test]
    fn empty_allowlist_allows_all() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: cmd\n    kind: run_command\n    program: echo\n    args: [test]";
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        assert!(Runner::run(&mut rt, &cfg, BTreeMap::new()).is_ok());
    }
}

mod split_sections {
    use super::*;

    #[test]
    fn extracts_multiple_sections() {
        let yaml = r#"
version: 1
name: test
steps:
  - id: input
    kind: write_file
    path: in.txt
    content: |
      TITLE:
      My Title

      DESC:
      My Desc
  - id: split
    kind: split_sections
    input_path: in.txt
    outputs:
      - heading: "TITLE:"
        path: title.txt
      - heading: "DESC:"
        path: desc.txt
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert_eq!(rt.read_text("title.txt").unwrap().trim(), "My Title");
        assert_eq!(rt.read_text("desc.txt").unwrap().trim(), "My Desc");
    }
}
