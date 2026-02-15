use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use vwf_core::runtime::{DryRunRuntime, FsRuntime, MockLlmClient};
use vwf_core::{Runner, WorkflowConfig};

#[derive(Parser, Debug)]
#[command(name = "vwf", version, about = "Video Workflow Framework CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run a workflow YAML file.
    Run {
        /// Path to workflow YAML.
        workflow: PathBuf,
        /// Work directory (outputs, artifacts).
        #[arg(long)]
        workdir: PathBuf,
        /// Extra vars: --var key=value
        #[arg(long="var", value_parser=parse_key_val, num_args=0..)]
        vars: Vec<(String, String)>,
        /// Dry-run: do not write files or execute commands.
        #[arg(long)]
        dry_run: bool,
        /// Allow executable (repeatable). Example: --allow ffmpeg --allow vhs
        #[arg(long="allow", num_args=0..)]
        allow: Vec<String>,
        /// Use mock LLM echo (default) or canned. Example: --mock-llm-canned "hello"
        #[arg(long)]
        mock_llm_canned: Option<String>,
    },
    /// Print a workflow after parsing (debug).
    Show { workflow: PathBuf },
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("expected key=value".into());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Show { workflow } => {
            let text = std::fs::read_to_string(&workflow)
                .with_context(|| format!("read {}", workflow.display()))?;
            let cfg: WorkflowConfig = serde_yaml::from_str(&text).context("parse workflow yaml")?;
            println!("{}", serde_json::to_string_pretty(&cfg)?);
            Ok(())
        }
        Cmd::Run {
            workflow,
            workdir,
            vars,
            dry_run,
            allow,
            mock_llm_canned,
        } => {
            let text = std::fs::read_to_string(&workflow)
                .with_context(|| format!("read {}", workflow.display()))?;
            let cfg: WorkflowConfig = serde_yaml::from_str(&text).context("parse workflow yaml")?;

            let extra_vars: BTreeMap<String, String> = vars.into_iter().collect();

            let llm: Box<dyn vwf_core::runtime::LlmClient> = match mock_llm_canned {
                Some(s) => Box::new(MockLlmClient::canned(s)),
                None => Box::new(MockLlmClient::echo()),
            };

            if dry_run {
                let mut rt = DryRunRuntime::new(&workdir, llm);
                let rep = Runner::run(&mut rt, &cfg, extra_vars)?;
                println!("{}", serde_json::to_string_pretty(&rep)?);
                println!("--- planned dirs ---");
                for d in rt.planned_dirs {
                    println!("{d}");
                }
                println!("--- planned writes ---");
                for (p, _) in rt.planned_writes {
                    println!("{p}");
                }
                Ok(())
            } else {
                std::fs::create_dir_all(&workdir)
                    .with_context(|| format!("create {}", workdir.display()))?;
                let mut rt = FsRuntime::new(&workdir, llm);
                rt.command_allowlist = allow.into_iter().collect::<BTreeSet<_>>();
                let rep = Runner::run(&mut rt, &cfg, extra_vars)?;
                // Always write run manifest
                let manifest_path = workdir.join("run.json");
                std::fs::write(&manifest_path, serde_json::to_vec_pretty(&rep)?)?;
                println!("Wrote {}", manifest_path.display());
                Ok(())
            }
        }
    }
}
