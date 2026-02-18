//! Workflow configuration parsing for VWF.

mod step;
mod workflow;

pub use step::{StepConfig, StepKind};
pub use workflow::{Segment, SegmentType, WorkflowConfig};
