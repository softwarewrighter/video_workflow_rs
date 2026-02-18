//! Handler for llm_generate step kind.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::context::StepCtx;
use vwf_runtime::LlmReq;

#[derive(Deserialize)]
struct Payload {
    system: String,
    user_prompt_path: String,
    output_path: String,
    provider: String,
    #[serde(default)]
    mock_response: Option<String>,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode llm_generate"))?;
    let system = ctx.render(&p.system)?;
    let user = read_user_prompt(ctx, &p.user_prompt_path)?;
    let user = inject_mock_response(user, p.mock_response);
    let provider = ctx.render(&p.provider)?;
    let req = LlmReq {
        system,
        user,
        provider,
    };
    let resp = ctx.rt.llm().generate(req)?;
    let output_path = ctx.render(&p.output_path)?;
    ctx.rt.write_text(&output_path, &resp)
}

fn read_user_prompt(ctx: &mut StepCtx<'_>, path: &str) -> Result<String> {
    let path = ctx.render(path)?;
    ctx.rt.read_text(&path)
}

fn inject_mock_response(user: String, mock: Option<String>) -> String {
    match mock {
        Some(m) => m,
        None => user,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use vwf_runtime::{DryRunRuntime, MockLlmClient};

    #[test]
    fn generates_llm_output() {
        let mut rt = DryRunRuntime::new("/tmp", Box::new(MockLlmClient::canned("response")));
        rt.planned_writes
            .push(("prompt.txt".into(), "user prompt".into()));
        let vars = BTreeMap::new();
        let payload = serde_json::json!({
            "system": "sys",
            "user_prompt_path": "prompt.txt",
            "output_path": "out.txt",
            "provider": "mock"
        });
        let mut ctx = StepCtx::new(&mut rt, &vars, "test");
        execute(&mut ctx, &payload).unwrap();
        assert!(
            rt.planned_writes
                .iter()
                .any(|(p, c)| p == "out.txt" && c == "response")
        );
    }
}
