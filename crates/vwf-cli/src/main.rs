use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use vwf_core::runtime::{DryRunRuntime, FsRuntime, MockLlmClient};
use vwf_core::{Runner, WorkflowConfig};

const LONG_ABOUT: &str = "\
Video Workflow Framework CLI - Repeatable video production workflows.

AI CODING AGENT INSTRUCTIONS:

This CLI runs YAML-defined video production workflows.

USAGE FOR AI AGENTS:
  1. Use `vwf show <workflow.yaml>` to inspect workflow structure
  2. Use `vwf run <workflow.yaml> --workdir <dir> --dry-run` to preview
  3. Use `vwf run <workflow.yaml> --workdir <dir>` to execute

WORKFLOW STEPS: ensure_dirs, write_file, split_sections, run_command, llm_generate
";

#[derive(Parser, Debug)]
#[command(
    name = "vwf",
    version = version_string(),
    about = "Video Workflow Framework CLI",
    long_about = LONG_ABOUT
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run a workflow YAML file.
    Run(RunArgs),
    /// Print a workflow after parsing (debug).
    Show { workflow: PathBuf },
}

#[derive(Parser, Debug)]
struct RunArgs {
    /// Path to workflow YAML.
    workflow: PathBuf,
    /// Work directory (outputs, artifacts).
    #[arg(long)]
    workdir: PathBuf,
    /// Extra vars: --var key=value
    #[arg(long = "var", value_parser = parse_kv, num_args = 0..)]
    vars: Vec<(String, String)>,
    /// Dry-run: do not write files or execute commands.
    #[arg(long)]
    dry_run: bool,
    /// Allow executable (repeatable).
    #[arg(long = "allow", num_args = 0..)]
    allow: Vec<String>,
    /// Use mock LLM with canned response.
    #[arg(long)]
    mock_llm_canned: Option<String>,
}

fn version_string() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        "\nCopyright: 2025 Software Wrighter LLC",
        "\nLicense: MIT",
        "\nRepository: github.com/softwarewrighter/video_workflow_rs",
        "\nBuild Host: ", env!("TARGET"),
        "\nBuild Commit: ", env!("GIT_HASH"),
        "\nBuild Time: ", env!("BUILD_TIME"),
    )
}

fn parse_kv(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| "expected key=value".into())
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Show { workflow } => run_show(&workflow),
        Cmd::Run(args) => run_workflow(args),
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
}

fn run_show(workflow: &PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(workflow)
        .with_context(|| format!("read {}", workflow.display()))?;
    let cfg = WorkflowConfig::from_yaml(&text)?;
    println!("{}", serde_json::to_string_pretty(&cfg)?);
    Ok(())
}

fn run_workflow(args: RunArgs) -> Result<()> {
    let text = std::fs::read_to_string(&args.workflow)
        .with_context(|| format!("read {}", args.workflow.display()))?;
    let cfg = WorkflowConfig::from_yaml(&text)?;
    let extra_vars: BTreeMap<_, _> = args.vars.into_iter().collect();
    let llm = make_llm_client(args.mock_llm_canned);

    if args.dry_run {
        run_dry(&args.workdir, llm, &cfg, extra_vars)
    } else {
        run_real(&args.workdir, llm, &cfg, extra_vars, args.allow)
    }
}

fn make_llm_client(canned: Option<String>) -> Box<dyn vwf_core::runtime::LlmClient> {
    match canned {
        Some(s) => Box::new(MockLlmClient::canned(s)),
        None => Box::new(MockLlmClient::echo()),
    }
}

fn run_dry(
    workdir: &PathBuf,
    llm: Box<dyn vwf_core::runtime::LlmClient>,
    cfg: &WorkflowConfig,
    vars: BTreeMap<String, String>,
) -> Result<()> {
    let mut rt = DryRunRuntime::new(workdir, llm);
    let rep = Runner::run(&mut rt, cfg, vars)?;
    println!("{}", serde_json::to_string_pretty(&rep)?);
    print_dry_run_plan(&rt);
    Ok(())
}

fn print_dry_run_plan(rt: &DryRunRuntime) {
    println!("--- planned dirs ---");
    for d in &rt.planned_dirs {
        println!("{d}");
    }
    println!("--- planned writes ---");
    for (p, _) in &rt.planned_writes {
        println!("{p}");
    }
}

fn run_real(
    workdir: &PathBuf,
    llm: Box<dyn vwf_core::runtime::LlmClient>,
    cfg: &WorkflowConfig,
    vars: BTreeMap<String, String>,
    allow: Vec<String>,
) -> Result<()> {
    std::fs::create_dir_all(workdir)
        .with_context(|| format!("create {}", workdir.display()))?;
    let mut rt = FsRuntime::new(workdir, llm);
    rt.command_allowlist = allow.into_iter().collect::<BTreeSet<_>>();
    let rep = Runner::run(&mut rt, cfg, vars)?;
    write_manifest(workdir, &rep)
}

fn write_manifest(workdir: &PathBuf, rep: &vwf_core::RunReport) -> Result<()> {
    let path = workdir.join("run.json");
    std::fs::write(&path, serde_json::to_vec_pretty(rep)?)?;
    println!("Wrote {}", path.display());
    Ok(())
}
