//! Workflow state management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Artifact, ArtifactId, Task, TaskId};

use super::CheckpointStatus;

/// Complete workflow state, persisted to state.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_name: String,
    pub version: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub inputs: BTreeMap<String, serde_json::Value>,
    pub tasks: BTreeMap<TaskId, Task>,
    pub artifacts: BTreeMap<ArtifactId, Artifact>,
    pub checkpoints: BTreeMap<String, CheckpointStatus>,
    pub complete: bool,
    pub error: Option<String>,
}

impl WorkflowState {
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

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
        self.updated_at = Utc::now();
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.insert(artifact.id.clone(), artifact);
        self.updated_at = Utc::now();
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }
}
