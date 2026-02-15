use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub vars: std::collections::BTreeMap<String, String>,
    pub steps: Vec<StepConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub id: String,
    pub kind: StepKind,

    // Payload (flattened) — each kind has its own fields.
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    EnsureDirs,
    WriteFile,
    SplitSections,
    RunCommand,
    LlmGenerate,
}

impl WorkflowConfig {
    /// Parse and validate a workflow from YAML string.
    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        let cfg: Self = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow::anyhow!("Failed to parse workflow YAML: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Validate the workflow configuration.
    pub fn validate(&self) -> anyhow::Result<()> {
        // Check for duplicate step IDs
        let mut seen_ids = std::collections::HashSet::new();
        for step in &self.steps {
            if !seen_ids.insert(&step.id) {
                anyhow::bail!("Duplicate step id: `{}`", step.id);
            }
            if step.id.is_empty() {
                anyhow::bail!("Step id cannot be empty");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            err.contains("unknown_kind") || err.contains("unknown variant"),
            "Error should mention the unknown kind: {err}"
        );
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
        assert!(
            err.contains("Duplicate step id") && err.contains("same"),
            "Error should mention duplicate id: {err}"
        );
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
        assert!(
            err.contains("empty"),
            "Error should mention empty id: {err}"
        );
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
}
