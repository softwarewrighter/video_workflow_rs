//! Workflow state persistence.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Artifact, ArtifactId, Task, TaskId};

/// Checkpoint status for user review points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStatus {
    pub name: String,
    pub message: String,
    pub reached_at: DateTime<Utc>,
    pub approved: bool,
    pub approved_at: Option<DateTime<Utc>>,
}

/// Complete workflow state, persisted to state.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow name.
    pub workflow_name: String,

    /// Workflow version.
    pub version: u32,

    /// When workflow started.
    pub started_at: DateTime<Utc>,

    /// When workflow last updated.
    pub updated_at: DateTime<Utc>,

    /// Input variables.
    pub inputs: BTreeMap<String, serde_json::Value>,

    /// All tasks in the workflow.
    pub tasks: BTreeMap<TaskId, Task>,

    /// All artifacts (produced and consumed).
    pub artifacts: BTreeMap<ArtifactId, Artifact>,

    /// Checkpoint statuses.
    pub checkpoints: BTreeMap<String, CheckpointStatus>,

    /// Is workflow complete?
    pub complete: bool,

    /// Error if workflow failed.
    pub error: Option<String>,
}

impl WorkflowState {
    /// Create new workflow state.
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        let now = Utc::now();
        Self {
            workflow_name: name.into(),
            version,
            started_at: now,
            updated_at: now,
            inputs: BTreeMap::new(),
            tasks: BTreeMap::new(),
            artifacts: BTreeMap::new(),
            checkpoints: BTreeMap::new(),
            complete: false,
            error: None,
        }
    }

    /// Add a task to the workflow.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
        self.updated_at = Utc::now();
    }

    /// Add an artifact to the workflow.
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.insert(artifact.id.clone(), artifact);
        self.updated_at = Utc::now();
    }

    /// Get a task by ID.
    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    /// Get a mutable task by ID.
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    /// Get an artifact by ID.
    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    /// Get tasks that are ready to run.
    pub fn ready_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.is_ready())
            .collect()
    }

    /// Get tasks that are blocked.
    pub fn blocked_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| matches!(t.status, super::TaskStatus::Blocked { .. }))
            .collect()
    }

    /// Check if a checkpoint is pending approval.
    pub fn checkpoint_pending(&self, name: &str) -> bool {
        self.checkpoints
            .get(name)
            .map(|c| !c.approved)
            .unwrap_or(false)
    }

    /// Approve a checkpoint.
    pub fn approve_checkpoint(&mut self, name: &str) -> Result<()> {
        let checkpoint = self
            .checkpoints
            .get_mut(name)
            .context(format!("Checkpoint not found: {name}"))?;
        checkpoint.approved = true;
        checkpoint.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Count completed tasks.
    pub fn completed_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_complete()).count()
    }

    /// Count total tasks.
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Calculate progress percentage.
    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }
        (self.completed_count() as f64 / self.total_count() as f64) * 100.0
    }
}

/// State persistence to filesystem.
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    /// Create a new state store at the given path.
    pub fn new(workdir: impl AsRef<Path>) -> Self {
        Self {
            path: workdir.as_ref().join("state.json"),
        }
    }

    /// Load state from disk, or return None if not found.
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

    /// Save state to disk.
    pub fn save(&self, state: &WorkflowState) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        Ok(())
    }

    /// Check if state file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn state_persistence() {
        let tmp = TempDir::new().unwrap();
        let store = StateStore::new(tmp.path());

        // Initially no state
        assert!(!store.exists());
        assert!(store.load().unwrap().is_none());

        // Create and save state
        let mut state = WorkflowState::new("test_workflow", 1);
        state.inputs.insert("episode".to_string(), serde_json::json!(17));

        let task = Task::new("generate_text", "llm_generate");
        state.add_task(task);

        store.save(&state).unwrap();
        assert!(store.exists());

        // Load and verify
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.workflow_name, "test_workflow");
        assert_eq!(loaded.version, 1);
        assert!(loaded.tasks.contains_key("generate_text"));
    }

    #[test]
    fn checkpoint_approval() {
        let mut state = WorkflowState::new("test", 1);

        state.checkpoints.insert(
            "review_text".to_string(),
            CheckpointStatus {
                name: "review_text".to_string(),
                message: "Review the text files".to_string(),
                reached_at: Utc::now(),
                approved: false,
                approved_at: None,
            },
        );

        assert!(state.checkpoint_pending("review_text"));

        state.approve_checkpoint("review_text").unwrap();

        assert!(!state.checkpoint_pending("review_text"));
    }
}
