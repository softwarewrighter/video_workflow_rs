//! Handler for run_command step kind.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::context::StepCtx;
use crate::runtime::CmdOut;

#[derive(Deserialize)]
struct Payload {
    program: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    capture_path: Option<String>,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode run_command"))?;
    let program = ctx.render(&p.program)?;
    let args = render_args(ctx, &p.args)?;
    let cwd = render_cwd(ctx, &p.cwd)?;
    let out = ctx.rt.run_command(&program, &args, cwd.as_deref())?;
    if let Some(cap) = p.capture_path {
        write_capture(ctx, &cap, &out)?;
    }
    Ok(())
}

fn render_args(ctx: &StepCtx<'_>, args: &[String]) -> Result<Vec<String>> {
    args.iter().map(|a| ctx.render(a)).collect()
}

fn render_cwd(ctx: &StepCtx<'_>, cwd: &Option<String>) -> Result<Option<String>> {
    match cwd {
        Some(c) => Ok(Some(ctx.render(c)?)),
        None => Ok(None),
    }
}

fn write_capture(ctx: &mut StepCtx<'_>, path: &str, out: &CmdOut) -> Result<()> {
    let path = ctx.render(path)?;
    let content = format!("status: {}\n\nstdout:\n{}\n\nstderr:\n{}\n", out.status, out.stdout, out.stderr);
    ctx.rt.write_text(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{DryRunRuntime, MockLlmClient};
    use std::collections::BTreeMap;

    #[test]
    fn captures_command_output() {
        let mut rt = DryRunRuntime::new("/tmp", Box::new(MockLlmClient::echo()));
        let vars = BTreeMap::new();
        let payload = serde_json::json!({"program": "echo", "args": ["hi"], "capture_path": "out.txt"});
        let mut ctx = StepCtx::new(&mut rt, &vars, "test");
        execute(&mut ctx, &payload).unwrap();
        assert!(!rt.planned_writes.is_empty());
    }
}
