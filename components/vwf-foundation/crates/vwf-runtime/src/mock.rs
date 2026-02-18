//! Mock LLM client for testing.

use anyhow::Result;

use super::traits::{LlmClient, LlmReq};

/// Mock LLM: returns a configured response or echoes input.
pub struct MockLlmClient {
    pub canned: Option<String>,
}

impl MockLlmClient {
    pub fn canned(s: impl Into<String>) -> Self {
        Self {
            canned: Some(s.into()),
        }
    }

    pub fn echo() -> Self {
        Self { canned: None }
    }
}

impl LlmClient for MockLlmClient {
    fn generate(&mut self, req: LlmReq) -> Result<String> {
        Ok(self
            .canned
            .clone()
            .unwrap_or_else(|| format!("SYSTEM:\n{}\n\nUSER:\n{}", req.system, req.user)))
    }
}
