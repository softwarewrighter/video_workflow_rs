//! Workflow execution commands.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{Context, Result};

use vwf_core::{DryRunRuntime, FsRuntime, LlmClient, MockLlmClient, RunOptions, Runner, RunReport, WorkflowConfig};

pub fn show(workflow: &PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(workflow).with_context(|| format!("read {}", workflow.display()))?;
    let cfg = WorkflowConfig::from_yaml(&text)?;
    println!("{}", serde_json::to_string_pretty(&cfg)?);
    Ok(())
}

pub fn execute(workflow: &PathBuf, workdir: &PathBuf, vars: Vec<(String, String)>, dry_run: bool, resume: bool, allow: Vec<String>, mock_llm: Option<String>) -> Result<()> {
    let text = std::fs::read_to_string(workflow).with_context(|| format!("read {}", workflow.display()))?;
    let cfg = WorkflowConfig::from_yaml(&text)?;
    let extra_vars: BTreeMap<_, _> = vars.into_iter().collect();
    let llm: Box<dyn LlmClient> = match mock_llm {
        Some(s) => Box::new(MockLlmClient::canned(s)),
        None => Box::new(MockLlmClient::echo()),
    };
    let opts = RunOptions { resume };
    if dry_run { run_dry(workdir, llm, &cfg, extra_vars) } else { run_real(workdir, llm, &cfg, extra_vars, allow, opts) }
}

fn run_dry(workdir: &PathBuf, llm: Box<dyn LlmClient>, cfg: &WorkflowConfig, vars: BTreeMap<String, String>) -> Result<()> {
    let mut rt = DryRunRuntime::new(workdir, llm);
    let rep = Runner::run(&mut rt, cfg, vars)?;
    println!("{}", serde_json::to_string_pretty(&rep)?);
    println!("--- planned dirs ---\n{}", rt.planned_dirs.join("\n"));
    println!("--- planned writes ---\n{}", rt.planned_writes.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>().join("\n"));
    Ok(())
}

fn run_real(workdir: &PathBuf, llm: Box<dyn LlmClient>, cfg: &WorkflowConfig, vars: BTreeMap<String, String>, allow: Vec<String>, opts: RunOptions) -> Result<()> {
    std::fs::create_dir_all(workdir).with_context(|| format!("create {}", workdir.display()))?;
    let mut rt = FsRuntime::new(workdir, llm);
    rt.command_allowlist = allow.into_iter().collect::<BTreeSet<_>>();
    let rep = Runner::run_with_options(&mut rt, cfg, vars, opts)?;
    write_manifest(workdir, &rep)
}

fn write_manifest(workdir: &PathBuf, rep: &RunReport) -> Result<()> {
    let path = workdir.join("run.json");
    std::fs::write(&path, serde_json::to_vec_pretty(rep)?)?;
    println!("Wrote {}", path.display());
    Ok(())
}
