//! Handler for write_file step kind.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    path: String,
    content: String,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode write_file"))?;
    let path = ctx.render(&p.path)?;
    let content = ctx.render(&p.content)?;
    ctx.rt.write_text(&path, &content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use vwf_runtime::{DryRunRuntime, MockLlmClient};

    #[test]
    fn writes_file_with_vars() {
        let mut rt = DryRunRuntime::new("/tmp", Box::new(MockLlmClient::echo()));
        let mut vars = BTreeMap::new();
        vars.insert("name".into(), "world".into());
        let payload = serde_json::json!({"path": "out.txt", "content": "Hello {{name}}"});
        let mut ctx = StepCtx::new(&mut rt, &vars, "test");
        execute(&mut ctx, &payload).unwrap();
        assert_eq!(
            rt.planned_writes,
            vec![("out.txt".into(), "Hello world".into())]
        );
    }
}
