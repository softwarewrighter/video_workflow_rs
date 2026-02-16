//! State persistence to filesystem.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::state::WorkflowState;

/// State persistence to filesystem.
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn new(workdir: impl AsRef<Path>) -> Self {
        Self { path: workdir.as_ref().join("state.json") }
    }

    pub fn load(&self) -> Result<Option<WorkflowState>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))?;
        let state: WorkflowState = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", self.path.display()))?;
        Ok(Some(state))
    }

    pub fn save(&self, state: &WorkflowState) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}
