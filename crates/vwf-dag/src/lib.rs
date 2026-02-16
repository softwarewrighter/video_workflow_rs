//! DAG-based workflow engine with reactive scheduling.
//!
//! Core concepts:
//! - **Task**: Unit of work with inputs, outputs, and constraints
//! - **Artifact**: File with checksum for incremental builds
//! - **Scheduler**: Runs tasks when dependencies are satisfied
//! - **State**: Persisted workflow state for resume

mod artifact;
mod scheduler;
mod state;
mod task;

pub use artifact::{Artifact, ArtifactId, ArtifactStatus};
pub use scheduler::{Scheduler, SchedulerEvent};
pub use state::{WorkflowState, StateStore};
pub use task::{Task, TaskId, TaskStatus, Constraint, InputSpec, OutputSpec};
