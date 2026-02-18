//! Task definitions for DAG-based workflow.

use serde::{Deserialize, Serialize};

/// Unique identifier for a task.
pub type TaskId = String;

/// Status of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is waiting for dependencies.
    Blocked {
        #[serde(default)]
        waiting_on: Vec<TaskId>,
    },
    /// All dependencies satisfied, ready to run.
    Ready,
    /// Task is currently executing.
    Running,
    /// Task completed successfully.
    Complete,
    /// Task failed with error.
    Failed { error: String },
    /// Task was skipped (optional input missing and no placeholder).
    Skipped { reason: String },
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Blocked { waiting_on: vec![] }
    }
}

/// Specification for a task input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputSpec {
    /// Required artifact - task blocks until available.
    Required { artifact: String },
    /// Optional artifact - task runs with or without.
    Optional {
        artifact: String,
        #[serde(default)]
        default: Option<String>,
    },
    /// Placeholder - task runs with placeholder, re-runs when real arrives.
    Placeholder {
        artifact: String,
        placeholder_kind: PlaceholderKind,
    },
}

/// Types of placeholders for missing inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceholderKind {
    SolidColor { color: String },
    SilentAudio { duration_secs: f64 },
    StaticImage { image_path: String },
    Skip,
}

/// Specification for a task output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    pub artifact: String,
    #[serde(default = "output_primary_default")]
    pub primary: bool,
}

const fn output_primary_default() -> bool {
    true
}

/// Execution constraints for a task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Constraint {
    #[serde(default)]
    pub sequential_group: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub max_parallelism: Option<u32>,
}

/// A task is a unit of work in the workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub kind: String,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    #[serde(default)]
    pub constraints: Constraint,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default)]
    pub config: serde_json::Value,
}

impl Task {
    /// Create a new task with the given id and kind.
    pub fn new(id: impl Into<String>, kind: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            kind: kind.into(),
            inputs: vec![],
            outputs: vec![],
            constraints: Constraint::default(),
            status: TaskStatus::default(),
            config: serde_json::Value::Null,
        }
    }

    /// Add a required input artifact (builder pattern).
    pub fn with_input(mut self, artifact: impl Into<String>) -> Self {
        self.inputs.push(InputSpec::Required {
            artifact: artifact.into(),
        });
        self
    }

    /// Add an output artifact (builder pattern).
    pub fn with_output(mut self, artifact: impl Into<String>) -> Self {
        self.outputs.push(OutputSpec {
            artifact: artifact.into(),
            primary: true,
        });
        self
    }

    /// Check if task is ready to run.
    pub fn is_ready(&self) -> bool {
        matches!(self.status, TaskStatus::Ready)
    }

    /// Check if task is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Complete)
    }
}
