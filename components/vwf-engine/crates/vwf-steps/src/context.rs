//! Step execution context with dependency injection.

use anyhow::Result;
use std::collections::BTreeMap;

use vwf_runtime::Runtime;

/// Execution context passed to all step handlers.
pub struct StepCtx<'a> {
    pub rt: &'a mut dyn Runtime,
    pub vars: &'a BTreeMap<String, String>,
    pub step_id: &'a str,
}

impl<'a> StepCtx<'a> {
    pub fn new(
        rt: &'a mut dyn Runtime,
        vars: &'a BTreeMap<String, String>,
        step_id: &'a str,
    ) -> Self {
        Self { rt, vars, step_id }
    }

    pub fn render(&self, template: &str) -> Result<String> {
        vwf_render::render_template(template, self.vars)
    }

    pub fn error_context(&self, msg: &str) -> String {
        format!("step `{}` {}", self.step_id, msg)
    }
}
