//! VWF Core: workflow engine + step library.
//!
//! Design principle: **workflows are data**, runner is code.
//! All side effects are mediated by `Runtime` so tests can swap in fakes.

pub mod config;
pub mod engine;
pub mod render;
pub mod runtime;
pub mod steps;

pub use config::{StepConfig, WorkflowConfig};
pub use engine::{RunReport, Runner, StepStatus};
pub use runtime::{DryRunRuntime, FsRuntime, LlmClient, MockLlmClient, Runtime};
