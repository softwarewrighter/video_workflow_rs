//! VWF Core: workflow engine + step library.
//!
//! Design principle: **workflows are data**, runner is code.
//! All side effects are mediated by `Runtime` so tests can swap in fakes.
//!
//! ## Modules
//!
//! - `config`: Linear workflow YAML parsing
//! - `engine`: Linear step execution
//! - `dag`: Reactive DAG-based workflow engine (Phase 2)

pub mod config;
pub mod dag;
pub mod engine;
pub mod render;
pub mod runtime;
pub mod steps;

pub use config::{StepConfig, WorkflowConfig};
pub use engine::{RunReport, Runner, StepStatus};
pub use runtime::{DryRunRuntime, FsRuntime, LlmClient, MockLlmClient, Runtime};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[test]
    fn full_workflow_produces_manifest_and_artifacts() {
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
  - id: write_config
    kind: write_file
    path: "work/config.txt"
    content: "version=1"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        let report = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();

        // Check report
        assert_eq!(report.workflow_name, "integration_test");
        assert_eq!(report.steps.len(), 3);
        assert!(
            report
                .steps
                .iter()
                .all(|s| matches!(s.status, StepStatus::Ok))
        );

        // Check artifacts
        let greeting = rt.read_text("output/greeting.txt").unwrap();
        assert_eq!(greeting, "Hello, World!");

        let config = rt.read_text("work/config.txt").unwrap();
        assert_eq!(config, "version=1");
    }

    #[test]
    fn dry_run_does_not_mutate_filesystem() {
        let yaml = r#"
version: 1
name: dry_run_test
steps:
  - id: create_dir
    kind: ensure_dirs
    dirs: ["should_not_exist"]
  - id: write_file
    kind: write_file
    path: "should_not_exist/file.txt"
    content: "test"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = DryRunRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        let report = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();

        // Report should show steps completed
        assert_eq!(report.steps.len(), 2);
        assert!(
            report
                .steps
                .iter()
                .all(|s| matches!(s.status, StepStatus::Ok))
        );

        // But filesystem should be untouched
        assert!(!tmp.path().join("should_not_exist").exists());

        // Planned operations should be recorded
        assert!(rt.planned_dirs.contains(&"should_not_exist".to_string()));
        assert!(
            rt.planned_writes
                .iter()
                .any(|(p, _)| p == "should_not_exist/file.txt")
        );
    }

    #[test]
    fn var_override_takes_precedence() {
        let yaml = r#"
version: 1
name: var_override_test
vars:
  name: default
steps:
  - id: write
    kind: write_file
    path: "out.txt"
    content: "Hello, {{name}}!"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        let mut overrides = BTreeMap::new();
        overrides.insert("name".to_string(), "Override".to_string());

        Runner::run(&mut rt, &cfg, overrides).unwrap();

        let content = rt.read_text("out.txt").unwrap();
        assert_eq!(content, "Hello, Override!");
    }

    #[test]
    fn missing_var_fails_with_context() {
        let yaml = r#"
version: 1
name: missing_var_test
steps:
  - id: write
    kind: write_file
    path: "out.txt"
    content: "Hello, {{undefined_var}}!"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("undefined_var"),
            "Error should mention the missing var: {msg}"
        );
        assert!(
            msg.contains("write"),
            "Error should mention the step id: {msg}"
        );
    }

    #[test]
    fn step_failure_includes_step_id_in_error() {
        let yaml = r#"
version: 1
name: step_failure_test
steps:
  - id: split_missing_file
    kind: split_sections
    input_path: "nonexistent.txt"
    outputs:
      - heading: "SECTION:"
        path: "out.txt"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("split_missing_file"),
            "Error should mention step id: {msg}"
        );
    }

    #[test]
    fn llm_generate_with_mock_response() {
        let yaml = r#"
version: 1
name: llm_test
steps:
  - id: write_prompt
    kind: write_file
    path: "prompt.txt"
    content: "Generate a greeting"
  - id: llm_call
    kind: llm_generate
    system: "You are helpful"
    user_prompt_path: "prompt.txt"
    output_path: "response.txt"
    provider: mock
    mock_response: |
      GREETING:
      Hello from mock LLM!
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();

        let response = rt.read_text("response.txt").unwrap();
        assert!(
            response.contains("GREETING:"),
            "Response should contain mock content"
        );
    }

    #[test]
    fn run_command_without_allowlist_fails() {
        let yaml = r#"
version: 1
name: command_test
steps:
  - id: run_echo
    kind: run_command
    program: echo
    args: ["hello"]
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        // No commands in allowlist
        rt.command_allowlist = std::collections::BTreeSet::new();
        rt.command_allowlist.insert("other".to_string()); // Add something else

        let err = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("not allowed") || msg.contains("allowlist"),
            "Error should mention command not allowed: {msg}"
        );
        assert!(
            msg.contains("echo"),
            "Error should mention the command: {msg}"
        );
    }

    #[test]
    fn run_command_with_allowlist_succeeds() {
        let yaml = r#"
version: 1
name: command_allowed_test
steps:
  - id: run_echo
    kind: run_command
    program: echo
    args: ["hello", "world"]
    capture_path: "output.txt"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        rt.command_allowlist.insert("echo".to_string());

        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();

        let output = rt.read_text("output.txt").unwrap();
        assert!(
            output.contains("hello world") || output.contains("hello\nworld"),
            "Output should contain command result: {output}"
        );
    }

    #[test]
    fn run_command_empty_allowlist_allows_all() {
        // When allowlist is empty, all commands are allowed (for dev convenience)
        let yaml = r#"
version: 1
name: command_no_restrict_test
steps:
  - id: run_echo
    kind: run_command
    program: echo
    args: ["test"]
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        // Empty allowlist = no restrictions

        let result = Runner::run(&mut rt, &cfg, BTreeMap::new());
        assert!(result.is_ok(), "Empty allowlist should allow all commands");
    }

    #[test]
    fn split_sections_extracts_multiple_sections() {
        let yaml = r#"
version: 1
name: split_test
steps:
  - id: write_input
    kind: write_file
    path: "input.txt"
    content: |
      TITLE:
      My Video Title

      DESCRIPTION:
      A short description here.

      NARRATION:
      The actual narration text goes here.
  - id: split
    kind: split_sections
    input_path: "input.txt"
    outputs:
      - heading: "TITLE:"
        path: "title.txt"
      - heading: "DESCRIPTION:"
        path: "description.txt"
      - heading: "NARRATION:"
        path: "narration.txt"
"#;
        let cfg = WorkflowConfig::from_yaml(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));

        Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();

        assert_eq!(rt.read_text("title.txt").unwrap().trim(), "My Video Title");
        assert_eq!(
            rt.read_text("description.txt").unwrap().trim(),
            "A short description here."
        );
        assert_eq!(
            rt.read_text("narration.txt").unwrap().trim(),
            "The actual narration text goes here."
        );
    }
}
