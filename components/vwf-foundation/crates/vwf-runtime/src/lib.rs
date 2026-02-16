//! Runtime abstraction for workflow execution.

mod dry_run;
mod fs;
mod mock;
mod traits;
mod validate;

pub use dry_run::DryRunRuntime;
pub use fs::FsRuntime;
pub use mock::MockLlmClient;
pub use traits::{CmdOut, LlmClient, LlmReq, Runtime};
pub use validate::output_is_valid;

// Re-export legacy names for compatibility
pub use traits::CmdOut as CommandOutput;
pub use traits::LlmReq as LlmRequest;
