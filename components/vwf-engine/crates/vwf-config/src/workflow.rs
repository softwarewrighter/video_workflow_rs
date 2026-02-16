//! Workflow configuration.

use std::collections::{BTreeMap, HashSet};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use super::StepConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    pub steps: Vec<StepConfig>,
}

impl WorkflowConfig {
    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        let cfg: Self = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow::anyhow!("Failed to parse workflow YAML: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let mut seen_ids = HashSet::new();
        for step in &self.steps {
            if step.id.is_empty() {
                bail!("Step id cannot be empty");
            }
            if !seen_ids.insert(&step.id) {
                bail!("Duplicate step id: `{}`", step.id);
            }
        }
        Ok(())
    }
}
