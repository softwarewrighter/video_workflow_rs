//! Workflow engine: executes steps sequentially.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::time::Instant;
use uuid::Uuid;

use vwf_config::{StepConfig, WorkflowConfig};
use vwf_render::render_template;
use vwf_runtime::{output_is_valid, Runtime};
use vwf_steps::execute_step;

use super::report::{RunReport, StepReport, StepStatus};

/// Options for workflow execution.
#[derive(Default)]
pub struct RunOptions {
    /// Skip steps whose output_path already exists and is valid.
    pub resume: bool,
}

pub struct Runner;

impl Runner {
    pub fn run(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra: BTreeMap<String, String>) -> Result<RunReport> {
        Self::run_with_options(rt, cfg, extra, RunOptions::default())
    }

    pub fn run_with_options(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra: BTreeMap<String, String>, opts: RunOptions) -> Result<RunReport> {
        let run_id = Uuid::new_v4();
        let started_at = Utc::now();
        let mut vars = cfg.vars.clone();
        vars.extend(extra);
        execute_steps(rt, &vars, &cfg.steps, run_id, &cfg.name, started_at, &opts)
    }
}

fn execute_steps(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, steps: &[StepConfig], run_id: Uuid, name: &str, started: DateTime<Utc>, opts: &RunOptions) -> Result<RunReport> {
    let mut reports = Vec::new();
    for step in steps {
        if opts.resume && should_skip(rt, vars, step) {
            reports.push(skipped_report(step));
            continue;
        }
        let report = run_step(rt, vars, step);
        if matches!(report.status, StepStatus::Failed) {
            let step_id = report.id.clone();
            let error_msg = report.error.clone().unwrap_or_default();
            reports.push(report);
            let r = RunReport { run_id, workflow_name: name.into(), started_at: started, finished_at: Utc::now(), steps: reports, vars: vars.clone() };
            return Err(anyhow::anyhow!("Workflow failed at step `{step_id}`: {error_msg}")).context(serde_json::to_string_pretty(&r).unwrap_or_default());
        }
        reports.push(report);
    }
    Ok(RunReport { run_id, workflow_name: name.into(), started_at: started, finished_at: Utc::now(), steps: reports, vars: vars.clone() })
}

fn should_skip(rt: &dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> bool {
    let Some(ref output) = step.resume_output else { return false };
    let Ok(path) = render_template(output, vars) else { return false };
    let full_path = rt.workdir().join(&path);
    output_is_valid(&full_path)
}

fn skipped_report(step: &StepConfig) -> StepReport {
    let now = Utc::now();
    StepReport { id: step.id.clone(), kind: format!("{:?}", step.kind), status: StepStatus::Skipped, started_at: now, finished_at: now, error: None, duration_ms: 0 }
}

fn run_step(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> StepReport {
    let started = Utc::now();
    let t0 = Instant::now();
    let result = execute_step(rt, vars, step);
    StepReport {
        id: step.id.clone(),
        kind: format!("{:?}", step.kind),
        status: if result.is_ok() { StepStatus::Ok } else { StepStatus::Failed },
        started_at: started,
        finished_at: Utc::now(),
        error: result.err().map(|e| e.to_string()),
        duration_ms: t0.elapsed().as_millis(),
    }
}
