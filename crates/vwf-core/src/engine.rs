use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;
use uuid::Uuid;

use crate::config::WorkflowConfig;
use crate::runtime::Runtime;
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
#[serde(rename_all="snake_case")]
pub enum StepStatus { Ok, Skipped, Failed }

pub struct Runner;

impl Runner {
    pub fn run(rt: &mut dyn Runtime, cfg: &WorkflowConfig, extra_vars: BTreeMap<String,String>) -> Result<RunReport> {
        let started_at = Utc::now();
        let run_id = Uuid::new_v4();
        let mut vars = cfg.vars.clone();
        vars.extend(extra_vars);

        let mut steps_report = vec![];

        for step in &cfg.steps {
            let s_started = Utc::now();
            let t0 = Instant::now();
            let res = execute_step(rt, &vars, step);
            let dur = t0.elapsed().as_millis();
            let s_finished = Utc::now();

            match res {
                Ok(()) => steps_report.push(StepReport{
                    id: step.id.clone(),
                    kind: format!("{:?}", step.kind),
                    status: StepStatus::Ok,
                    started_at: s_started,
                    finished_at: s_finished,
                    error: None,
                    duration_ms: dur,
                }),
                Err(e) => {
                    steps_report.push(StepReport{
                        id: step.id.clone(),
                        kind: format!("{:?}", step.kind),
                        status: StepStatus::Failed,
                        started_at: s_started,
                        finished_at: s_finished,
                        error: Some(e.to_string()),
                        duration_ms: dur,
                    });
                    let report = RunReport{
                        run_id,
                        workflow_name: cfg.name.clone(),
                        started_at,
                        finished_at: Utc::now(),
                        steps: steps_report,
                        vars,
                    };
                    return Err(anyhow::anyhow!("Workflow failed at step `{}`: {}", step.id, e))
                        .context(serde_json::to_string_pretty(&report).unwrap_or_default());
                }
            }
        }

        Ok(RunReport{
            run_id,
            workflow_name: cfg.name.clone(),
            started_at,
            finished_at: Utc::now(),
            steps: steps_report,
            vars,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{FsRuntime, MockLlmClient};
    use tempfile::TempDir;

    #[test]
    fn runner_writes_files() {
        let cfg: WorkflowConfig = serde_yaml::from_str(r#"
version: 1
name: test
steps:
  - id: d
    kind: ensure_dirs
    dirs: ["work"]
  - id: w
    kind: write_file
    path: "work/hello.txt"
    content: "hi"
"#).unwrap();

        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        let rep = Runner::run(&mut rt, &cfg, BTreeMap::new()).unwrap();
        assert_eq!(rep.workflow_name, "test");
        let s = rt.read_text("work/hello.txt").unwrap();
        assert_eq!(s, "hi");
    }
}

