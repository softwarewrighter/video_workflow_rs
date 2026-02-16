//! Handler for ensure_dirs step kind.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    dirs: Vec<String>,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode ensure_dirs"))?;
    for d in p.dirs {
        ctx.rt.ensure_dir(&ctx.render(&d)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vwf_runtime::{DryRunRuntime, MockLlmClient};
    use std::collections::BTreeMap;

    #[test]
    fn creates_multiple_dirs() {
        let mut rt = DryRunRuntime::new("/tmp", Box::new(MockLlmClient::echo()));
        let vars = BTreeMap::new();
        let payload = serde_json::json!({"dirs": ["a", "b/c"]});
        let mut ctx = StepCtx::new(&mut rt, &vars, "test");
        execute(&mut ctx, &payload).unwrap();
        assert_eq!(rt.planned_dirs, vec!["a", "b/c"]);
    }
}
