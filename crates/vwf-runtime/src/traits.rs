//! Core runtime traits and types.

use anyhow::Result;
use std::path::Path;

/// Runtime mediates side effects so the engine can be tested.
pub trait Runtime {
    fn workdir(&self) -> &Path;
    fn ensure_dir(&mut self, rel: &str) -> Result<()>;
    fn write_text(&mut self, rel: &str, content: &str) -> Result<()>;
    fn read_text(&self, rel: &str) -> Result<String>;
    fn run_command(&mut self, prog: &str, args: &[String], cwd: Option<&str>) -> Result<CmdOut>;
    fn llm(&mut self) -> &mut dyn LlmClient;
}

/// Output from a shell command execution.
#[derive(Debug, Clone)]
pub struct CmdOut {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// LLM provider abstraction.
pub trait LlmClient {
    fn generate(&mut self, req: LlmReq) -> Result<String>;
}

/// LLM request parameters.
#[derive(Debug, Clone)]
pub struct LlmReq {
    pub system: String,
    pub user: String,
    pub provider: String,
}
