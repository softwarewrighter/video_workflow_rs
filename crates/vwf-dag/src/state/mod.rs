//! Workflow state persistence.

mod queries;
mod store;
mod workflow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use store::StateStore;
pub use workflow::WorkflowState;

/// Checkpoint status for user review points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStatus {
    pub name: String,
    pub message: String,
    pub reached_at: DateTime<Utc>,
    pub approved: bool,
    pub approved_at: Option<DateTime<Utc>>,
}
