//! Status types for tasks and artifacts.

use serde::{Deserialize, Serialize};

use super::TaskId;

/// Status of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Blocked { waiting_on: Vec<TaskId> },
    Ready,
    Running,
    Complete,
    Failed { error: String },
    Skipped { reason: String },
}

/// Status of an artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Missing,
    Placeholder,
    Ready,
    Invalidated,
}
