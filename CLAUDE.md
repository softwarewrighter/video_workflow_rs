# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

VWF (Video Workflow Framework) is a Rust-based workflow engine that eliminates "agent drift" in video production. The core principle: **workflows are data (YAML), runner is code**. All side effects go through traits for testability.

## CRITICAL: Dogfooding Required

**NEVER run ad hoc commands to produce video assets.** All video production work MUST go through the workflow engine:

1. **Use `workflow.yaml`** - Every operation (TTS, image generation, music, video assembly) must be a workflow step
2. **No manual ffmpeg/convert/etc.** - If you need a capability, add it to the workflow
3. **Missing capability?** - Implement it as a new step type in `vwf-steps`, test it, then use it
4. **Incremental builds** - Use `--resume` flag; steps with `resume_output` skip if output exists

```bash
# CORRECT: Run workflow with --resume for incremental builds
cargo run -p vwf-cli -- run projects/self/workflow.yaml --workdir projects/self --resume --allow bash --allow ffmpeg

# WRONG: Running ffmpeg directly
ffmpeg -i input.wav -i background.png output.mp4  # DO NOT DO THIS
```

**Why this matters:**
- Workflows are reproducible and auditable (`run.json` records every step)
- Incremental builds save time (only re-run changed steps)
- No "agent drift" - the workflow is the source of truth
- We eat our own dogfood - if VWF can't do it, we fix VWF

## Commands

Use the scripts in `scripts/` instead of running cargo directly:

```bash
# Build all components
./scripts/build.sh

# Test all crates + Python hygiene
./scripts/test.sh

# Lint (zero warnings required)
./scripts/lint.sh

# Format code
./scripts/fmt.sh

# Run workflow with resume (skips completed steps)
./scripts/run.sh projects/self/workflow.yaml --resume --allow bash --allow ffmpeg

# Dry run (preview only)
./scripts/run.sh projects/self/workflow.yaml --dry-run

# Run with variable override
./scripts/run.sh workflow.yaml --var project_name=Demo

# Debug workflow parsing
./scripts/show.sh examples/workflows/shorts_narration.yaml

# Check service availability
cd components/vwf-apps && cargo run -p vwf-cli -- services ../../projects/self/workflow.yaml

# Serve web UI (status viewer)
./scripts/serve-web.sh

# Setup Python environment
./scripts/setup-python.sh
```

## Architecture

### Workspace Structure

Two Cargo workspaces under `components/`:

**`components/vwf-engine/`** - Core library crates:
- **vwf-config**: YAML parsing, StepKind enum, WorkflowConfig
- **vwf-steps**: Step implementations (tts_generate, text_to_image, etc.)

**`components/vwf-apps/`** - Application crates:
- **vwf-cli**: Command-line interface (`vwf run`, `vwf show`)
- **vwf-web**: Yew/WASM UI skeleton (future)

Run CLI from vwf-apps workspace:
```bash
cd components/vwf-apps && cargo run -p vwf-cli -- run ../../projects/self/workflow.yaml --workdir ../../projects/self
```

### Core Abstractions

**Runtime trait** (`runtime.rs`): All I/O goes through this trait. Two implementations:
- `FsRuntime`: Real filesystem + command execution
- `DryRunRuntime`: Records planned operations without mutations

**LlmClient trait**: Mockable LLM adapter. `MockLlmClient` returns echo or canned responses.

**WorkflowConfig** (`config.rs`): Parsed from YAML with versioning, vars, and ordered steps.

**StepKind enum**: `ensure_dirs`, `write_file`, `split_sections`, `run_command`, `llm_generate`, `tts_generate`, `text_to_image`, `image_to_video`, `text_to_video`, `normalize_volume`, `whisper_transcribe`, `video_concat`, `audio_mix`, `create_slide`, `llm_audit`

### DAG Execution

The workflow engine uses DAG-based execution with the `depends_on` field:

- **Steps run as soon as dependencies are satisfied** - no artificial sequencing
- **Failed steps don't block unrelated work** - only steps that depend on failed steps are blocked
- **Clear status reporting** - shows ok, skipped, failed, and blocked steps with reasons
- **Cycle detection** - validates the DAG before execution to catch circular dependencies
- **Resume support** - use `--resume` to skip completed steps and continue from failures

Example workflow with dependencies:
```yaml
steps:
  - id: setup_dirs
    kind: ensure_dirs
    dirs: ["work/scripts", "work/audio"]

  - id: write_script
    kind: write_file
    depends_on: [setup_dirs]
    path: "work/scripts/narration.txt"
    content: "Hello world"

  - id: generate_audio
    kind: tts_generate
    depends_on: [write_script]  # Waits for script to be written
    script_path: "work/scripts/narration.txt"
    output_path: "work/audio/narration.wav"
```

### Data Flow

1. Parse YAML -> `WorkflowConfig`
2. `Runner::run()` iterates steps with timing
3. Each step executed via `execute_step()` in `steps.rs`
4. Template `{{var}}` substitution via `render.rs`
5. Produces `run.json` manifest with step reports

### Key Files

- `components/vwf-engine/crates/vwf-config/src/step.rs`: StepKind enum and step config
- `components/vwf-engine/crates/vwf-steps/src/`: Step implementations (one file per step type)
- `components/vwf-apps/crates/vwf-cli/src/main.rs`: CLI entry point with clap
- `projects/*/workflow.yaml`: Video project workflow definitions
- `docs/narration-style.md`: TTS narration guidelines (avoid acronyms, use phonetic spelling)

## TDD Approach

Follow red/green/refactor strictly. See `docs/TDD_PLAN.md` for test sequence:

1. Parse minimal YAML
2. Validate unknown step type -> error with step ID
3. Template rendering with missing var error
4. Step execution with mock runtime
5. Full workflow producing manifest

## Step Payload Pattern

Steps use serde flatten for payloads:
```rust
#[derive(Deserialize)]
struct Payload { dirs: Vec<String> }
let p: Payload = serde_json::from_value(step.payload.clone())?;
```

Each step defines its own inline Payload struct for type safety.

## Error Convention

Errors must include step ID:
```rust
.with_context(|| format!("step `{}` payload decode ensure_dirs", step.id))
```

## Command Safety

`run_command` requires explicit `--allow <program>` flags. Without allowlist, commands fail with remediation message.

## Adding New Step Types

When a workflow capability is missing, implement it as a new step type:

1. **Add variant to StepKind** in `vwf-config/src/step.rs`
2. **Create implementation** in `vwf-steps/src/<step_name>.rs`
3. **Register in lib.rs** - add to `vwf-steps/src/lib.rs` module list and match arm
4. **Test it** - write unit tests for the new step
5. **Use it** - add to workflow.yaml and run with `--resume`

Example step implementation pattern:
```rust
#[derive(Deserialize)]
struct Payload {
    input_path: String,
    output_path: String,
}

pub fn execute(ctx: &mut StepContext) -> Result<()> {
    let p: Payload = serde_json::from_value(ctx.step.payload.clone())?;
    let input = ctx.render(&p.input_path)?;
    let output = ctx.render(&p.output_path)?;
    // ... implementation
    Ok(())
}
```

## Python Package Management

**NEVER run `pip` or `pip3` directly.** Always use a virtual environment with `uv`:

```bash
# Create virtual environment (run from repo root)
uv venv

# Activate the environment
source .venv/bin/activate

# Install all workflow dependencies
uv pip install -r requirements.txt

# Verify installation
python -c "import requests; from gradio_client import Client; print('OK')"
```

**Key requirements:**
- `gradio-client` - for TTS generation via VoxCPM
- `requests` - for text-to-image via ComfyUI/FLUX.1
- `pillow` - for image processing

**Workflow Configuration:**
Steps that use Python (tts_generate, text_to_image, llm_audit) accept a `python_path` parameter.
Set this in your workflow vars to use the venv:

```yaml
vars:
  python_path: "/path/to/repo/.venv/bin/python"
```

This ensures reproducible environments and avoids polluting the system Python installation.
