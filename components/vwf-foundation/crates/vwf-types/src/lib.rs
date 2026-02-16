//! Shared types for VWF workflow system.

mod ids;
mod status;

pub use ids::{ArtifactId, TaskId};
pub use status::{ArtifactStatus, TaskStatus};
