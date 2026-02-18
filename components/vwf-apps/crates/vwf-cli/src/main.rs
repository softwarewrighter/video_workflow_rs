mod generate;
mod run;
mod services;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

const LONG_ABOUT: &str = "\
Video Workflow Framework CLI - Repeatable video production workflows.

AI CODING AGENT INSTRUCTIONS:

This CLI runs YAML-defined video production workflows.

USAGE FOR AI AGENTS:
  1. Use `vwf generate <project-dir>` to create a workflow from a brief
  2. Use `vwf show <workflow.yaml>` to inspect workflow structure
  3. Use `vwf run <workflow.yaml> --workdir <dir> --dry-run` to preview
  4. Use `vwf run <workflow.yaml> --workdir <dir>` to execute

WORKFLOW STEPS: ensure_dirs, write_file, split_sections, run_command, llm_generate, tts_generate,
                text_to_image, image_to_video, text_to_video, normalize_volume, audio_mix,
                video_concat, create_slide, whisper_transcribe

RESUME MODE:
  Use `--resume` to skip steps whose output files already exist and are valid.
  Useful after power outage or interrupted workflow.
";

#[derive(Parser, Debug)]
#[command(name = "vwf", version = version_string(), about = "Video Workflow Framework CLI", long_about = LONG_ABOUT)]
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
    /// Generate a workflow.yaml from a project brief using LLM.
    Generate(GenerateArgs),
    /// Check availability of services required by a workflow.
    Services { workflow: PathBuf },
}

#[derive(Parser, Debug)]
struct GenerateArgs {
    /// Project directory containing brief.txt
    project_dir: PathBuf,
    /// Ollama model to use (default: qwen2.5-coder:14b)
    #[arg(long, default_value = "qwen2.5-coder:14b")]
    model: String,
    /// Path to LLM context documents (default: auto-detect)
    #[arg(long)]
    context_dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct RunArgs {
    workflow: PathBuf,
    #[arg(long)]
    workdir: PathBuf,
    #[arg(long = "var", value_parser = parse_kv, num_args = 0..)]
    vars: Vec<(String, String)>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    resume: bool,
    #[arg(long = "allow", num_args = 0..)]
    allow: Vec<String>,
    #[arg(long)]
    mock_llm_canned: Option<String>,
    /// Ollama model for LLM generation (e.g., "qwen2.5-coder:14b", "gemma2:9b")
    #[arg(long)]
    llm_model: Option<String>,
}

fn version_string() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        "\nCopyright: 2025 Software Wrighter LLC",
        "\nLicense: MIT",
        "\nRepository: github.com/softwarewrighter/video_workflow_rs",
        "\nBuild Host: ",
        env!("TARGET"),
        "\nBuild Commit: ",
        env!("GIT_HASH"),
        "\nBuild Time: ",
        env!("BUILD_TIME")
    )
}

fn parse_kv(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.into(), v.into()))
        .ok_or_else(|| "expected key=value".into())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Show { workflow } => run::show(&workflow),
        Cmd::Run(args) => run::execute(
            &args.workflow,
            &args.workdir,
            args.vars,
            args.dry_run,
            args.resume,
            args.allow,
            args.mock_llm_canned,
            args.llm_model,
        ),
        Cmd::Generate(args) => {
            generate::generate(&args.project_dir, &args.model, args.context_dir.as_deref())
        }
        Cmd::Services { workflow } => services::check_services(&workflow),
    }
}
