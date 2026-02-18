//! Workflow engine orchestration for VWF.

mod engine;
mod report;

pub use engine::{RunOptions, Runner};
pub use report::{RunReport, StepReport, StepStatus};

// Re-export dependencies for convenience
pub use vwf_config::{StepConfig, StepKind, WorkflowConfig};
pub use vwf_render::render_template;
pub use vwf_runtime::{CmdOut, DryRunRuntime, FsRuntime, LlmClient, LlmReq, MockLlmClient, OllamaClient, Runtime};
