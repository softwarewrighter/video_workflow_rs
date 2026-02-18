//! Ollama LLM client implementation.
//!
//! Connects to local Ollama server for text generation.

use anyhow::{Context, Result};

use super::traits::{LlmClient, LlmReq};

/// Ollama LLM client.
pub struct OllamaClient {
    /// Ollama server URL (default: http://localhost:11434)
    pub server: String,
    /// Model to use (e.g., "qwen2.5-coder:14b", "gemma2:9b")
    pub model: String,
}

impl OllamaClient {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            server: "http://localhost:11434".to_string(),
            model: model.into(),
        }
    }

    pub fn with_server(mut self, server: impl Into<String>) -> Self {
        self.server = server.into();
        self
    }
}

impl LlmClient for OllamaClient {
    fn generate(&mut self, req: LlmReq) -> Result<String> {
        // Build the request payload
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": req.user,
            "system": req.system,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "num_predict": 2048
            }
        });

        println!("LLM Generate via Ollama:");
        println!("  Server: {}", self.server);
        println!("  Model: {}", self.model);
        println!("  System: {}...", req.system.chars().take(50).collect::<String>());
        println!("  Prompt: {}...", req.user.chars().take(50).collect::<String>());

        // Make HTTP request to Ollama
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/api/generate", self.server))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(300)) // 5 min timeout for long generations
            .send()
            .context("Failed to connect to Ollama server")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Ollama request failed ({}): {}", status, body);
        }

        let result: serde_json::Value = response.json().context("Failed to parse Ollama response")?;

        let text = result["response"]
            .as_str()
            .context("No 'response' field in Ollama output")?
            .to_string();

        println!("  Response: {} chars", text.len());

        Ok(text)
    }
}
