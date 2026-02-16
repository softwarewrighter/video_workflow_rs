//! Workflow engine: executes steps sequentially.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::time::Instant;
use uuid::Uuid;

use vwf_config::{StepConfig, WorkflowConfig};
use vwf_runtime::Runtime;
use vwf_steps::execute_step;

use super::report::{RunReport, StepReport, StepStatus};

pub struct Runner;

impl Runner {
    pub fn run(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra: BTreeMap<String, String>) -> Result<RunReport> {
        run_workflow(rt, cfg, extra)
    }
}

pub fn run_workflow(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra: BTreeMap<String, String>) -> Result<RunReport> {
    let run_id = Uuid::new_v4();
    let started_at = Utc::now();
    let vars = merge_vars(&cfg.vars, extra);
    execute_steps(rt, &vars, &cfg.steps, run_id, &cfg.name, started_at)
}

fn merge_vars(base: &BTreeMap<String, String>, extra: BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut vars = base.clone();
    vars.extend(extra);
    vars
}

fn execute_steps(
    rt: &mut dyn Runtime,
    vars: &BTreeMap<String, String>,
    steps: &[StepConfig],
    run_id: Uuid,
    name: &str,
    started: DateTime<Utc>,
) -> Result<RunReport> {
    let mut reports = Vec::new();
    for step in steps {
        let report = run_step(rt, vars, step);
        if matches!(report.status, StepStatus::Failed) {
            return fail_workflow(run_id, name, started, reports, report, vars);
        }
        reports.push(report);
    }
    Ok(make_report(run_id, name, started, reports, vars))
}

fn run_step(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> StepReport {
    let started = Utc::now();
    let t0 = Instant::now();
    let result = execute_step(rt, vars, step);
    step_report(step, started, t0.elapsed().as_millis(), result)
}

fn step_report(step: &StepConfig, started: DateTime<Utc>, dur: u128, result: Result<()>) -> StepReport {
    StepReport {
        id: step.id.clone(),
        kind: format!("{:?}", step.kind),
        status: if result.is_ok() { StepStatus::Ok } else { StepStatus::Failed },
        started_at: started,
        finished_at: Utc::now(),
        error: result.err().map(|e| e.to_string()),
        duration_ms: dur,
    }
}

fn make_report(run_id: Uuid, name: &str, started: DateTime<Utc>, steps: Vec<StepReport>, vars: &BTreeMap<String, String>) -> RunReport {
    RunReport { run_id, workflow_name: name.to_string(), started_at: started, finished_at: Utc::now(), steps, vars: vars.clone() }
}

fn fail_workflow(run_id: Uuid, name: &str, started: DateTime<Utc>, mut steps: Vec<StepReport>, failed: StepReport, vars: &BTreeMap<String, String>) -> Result<RunReport> {
    let step_id = failed.id.clone();
    let error_msg = failed.error.clone().unwrap_or_default();
    steps.push(failed);
    let report = make_report(run_id, name, started, steps, vars);
    Err(anyhow::anyhow!("Workflow failed at step `{}`: {}", step_id, error_msg))
        .context(serde_json::to_string_pretty(&report).unwrap_or_default())
}
