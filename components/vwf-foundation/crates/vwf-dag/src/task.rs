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
        waiting_on: Vec<TaskId>,
    },
    /// All dependencies satisfied, ready to run.
    Ready,
    /// Task is currently executing.
    Running,
    /// Task completed successfully.
    Complete,
    /// Task failed with error.
    Failed {
        error: String,
    },
    /// Task was skipped (optional input missing and no placeholder).
    Skipped {
        reason: String,
    },
}

/// Specification for a task input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputSpec {
    /// Required artifact - task blocks until available.
    Required {
        artifact: String,
    },
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
    /// Solid color for missing images.
    SolidColor { color: String },
    /// Silent audio with specified duration.
    SilentAudio { duration_secs: f64 },
    /// Static image for missing video.
    StaticImage { image_path: String },
    /// Skip this input entirely.
    Skip,
}

/// Specification for a task output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    /// Output artifact path (may contain patterns like *.mp4).
    pub artifact: String,
    /// Is this a primary output (vs. side effect)?
    #[serde(default = "default_true")]
    pub primary: bool,
}

fn default_true() -> bool {
    true
}

/// Execution constraints for a task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Constraint {
    /// Sequential group - only one task in this group runs at a time.
    /// Example: "tts" for GPU-bound TTS generation.
    #[serde(default)]
    pub sequential_group: Option<String>,

    /// Resource requirement (e.g., "gpu").
    #[serde(default)]
    pub resource: Option<String>,

    /// Maximum parallelism for this task type.
    #[serde(default)]
    pub max_parallelism: Option<u32>,
}

/// A task is a unit of work in the workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier.
    pub id: TaskId,

    /// Human-readable name.
    pub name: String,

    /// Task kind (maps to executor).
    pub kind: String,

    /// Input specifications.
    pub inputs: Vec<InputSpec>,

    /// Output specifications.
    pub outputs: Vec<OutputSpec>,

    /// Execution constraints.
    #[serde(default)]
    pub constraints: Constraint,

    /// Current status.
    #[serde(default = "default_blocked")]
    pub status: TaskStatus,

    /// Task-specific configuration (passed to executor).
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_blocked() -> TaskStatus {
    TaskStatus::Blocked { waiting_on: vec![] }
}

impl Task {
    /// Create a new task.
    pub fn new(id: impl Into<String>, kind: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            kind: kind.into(),
            inputs: vec![],
            outputs: vec![],
            constraints: Constraint::default(),
            status: TaskStatus::Blocked { waiting_on: vec![] },
            config: serde_json::Value::Null,
        }
    }

    /// Add a required input.
    pub fn with_required_input(mut self, artifact: impl Into<String>) -> Self {
        self.inputs.push(InputSpec::Required {
            artifact: artifact.into(),
        });
        self
    }

    /// Add a placeholder input.
    pub fn with_placeholder_input(
        mut self,
        artifact: impl Into<String>,
        placeholder: PlaceholderKind,
    ) -> Self {
        self.inputs.push(InputSpec::Placeholder {
            artifact: artifact.into(),
            placeholder_kind: placeholder,
        });
        self
    }

    /// Add an output.
    pub fn with_output(mut self, artifact: impl Into<String>) -> Self {
        self.outputs.push(OutputSpec {
            artifact: artifact.into(),
            primary: true,
        });
        self
    }

    /// Set sequential constraint.
    pub fn sequential(mut self, group: impl Into<String>) -> Self {
        self.constraints.sequential_group = Some(group.into());
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

    /// Get IDs of tasks this task is waiting on.
    pub fn waiting_on(&self) -> &[TaskId] {
        match &self.status {
            TaskStatus::Blocked { waiting_on } => waiting_on,
            _ => &[],
        }
    }
}
