//! Step handlers for workflow execution.

mod context;
mod ensure_dirs;
mod llm_generate;
mod run_command;
mod split_sections;
mod write_file;

use anyhow::Result;
use std::collections::BTreeMap;

use crate::config::{StepConfig, StepKind};
use vwf_runtime::Runtime;
use context::StepCtx;

/// Execute a single step with dependency injection via Runtime trait.
pub fn execute_step(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> Result<()> {
    let mut ctx = StepCtx::new(rt, vars, &step.id);
    dispatch(&mut ctx, &step.kind, &step.payload)
}

fn dispatch(ctx: &mut StepCtx<'_>, kind: &StepKind, payload: &serde_json::Value) -> Result<()> {
    match kind {
        StepKind::EnsureDirs => ensure_dirs::execute(ctx, payload),
        StepKind::WriteFile => write_file::execute(ctx, payload),
        StepKind::SplitSections => split_sections::execute(ctx, payload),
        StepKind::RunCommand => run_command::execute(ctx, payload),
        StepKind::LlmGenerate => llm_generate::execute(ctx, payload),
    }
}
