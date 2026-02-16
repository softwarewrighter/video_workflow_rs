//! Workflow engine: executes steps sequentially.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;
use uuid::Uuid;

use crate::config::{StepConfig, WorkflowConfig};
use vwf_runtime::Runtime;
use crate::steps::execute_step;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub run_id: Uuid,
    pub workflow_name: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub steps: Vec<StepReport>,
    pub vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepReport {
    pub id: String,
    pub kind: String,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub error: Option<String>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Ok,
    Skipped,
    Failed,
}

pub struct Runner;

impl Runner {
    pub fn run(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra: BTreeMap<String, String>) -> Result<RunReport> {
        run_workflow(rt, cfg, extra)
    }
}

/// Pure function to run a workflow.
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
    Ok(make_run_report(run_id, name, started, reports, vars))
}

fn run_step(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> StepReport {
    let started = Utc::now();
    let t0 = Instant::now();
    let result = execute_step(rt, vars, step);
    make_step_report(step, started, t0.elapsed().as_millis(), result)
}

fn make_step_report(step: &StepConfig, started: DateTime<Utc>, dur: u128, result: Result<()>) -> StepReport {
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

fn make_run_report(run_id: Uuid, name: &str, started: DateTime<Utc>, steps: Vec<StepReport>, vars: &BTreeMap<String, String>) -> RunReport {
    RunReport { run_id, workflow_name: name.to_string(), started_at: started, finished_at: Utc::now(), steps, vars: vars.clone() }
}

fn fail_workflow(run_id: Uuid, name: &str, started: DateTime<Utc>, mut steps: Vec<StepReport>, failed: StepReport, vars: &BTreeMap<String, String>) -> Result<RunReport> {
    let step_id = failed.id.clone();
    let error_msg = failed.error.clone().unwrap_or_default();
    steps.push(failed);
    let report = make_run_report(run_id, name, started, steps, vars);
    Err(anyhow::anyhow!("Workflow failed at step `{}`: {}", step_id, error_msg))
        .context(serde_json::to_string_pretty(&report).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vwf_runtime::{FsRuntime, MockLlmClient};
    use tempfile::TempDir;

    #[test]
    fn runner_writes_files() {
        let yaml = "version: 1\nname: test\nsteps:\n  - id: d\n    kind: ensure_dirs\n    dirs: [\"work\"]\n  - id: w\n    kind: write_file\n    path: \"work/hello.txt\"\n    content: \"hi\"";
        let cfg: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let rep = run_workflow(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert_eq!(rep.workflow_name, "test");
        assert_eq!(rt.read_text("work/hello.txt").unwrap(), "hi");
    }
}
