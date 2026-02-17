# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

VWF (Video Workflow Framework) is a Rust-based workflow engine that eliminates "agent drift" in video production. The core principle: **workflows are data (YAML), runner is code**. All side effects go through traits for testability.

## Commands

```bash
# Build
cargo build

# Test all crates
cargo test

# Test single crate
cargo test -p vwf-core

# Lint (zero warnings required)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Run workflow (real execution)
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo

# Dry run (preview only)
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo --dry-run

# Run with variable override
cargo run -p vwf-cli -- run workflow.yaml --workdir work --var project_name=Demo

# Allow specific commands
cargo run -p vwf-cli -- run workflow.yaml --workdir work --allow ffmpeg --allow vhs

# Debug workflow parsing
cargo run -p vwf-cli -- show examples/workflows/shorts_narration.yaml
```

## Architecture

### Workspace Structure

- **vwf-core**: Engine + config parsing + step implementations + Runtime trait
- **vwf-cli**: Command-line interface (`vwf run`, `vwf show`)
- **vwf-web**: Yew/WASM UI skeleton (future)

### Core Abstractions

**Runtime trait** (`runtime.rs`): All I/O goes through this trait. Two implementations:
- `FsRuntime`: Real filesystem + command execution
- `DryRunRuntime`: Records planned operations without mutations

**LlmClient trait**: Mockable LLM adapter. `MockLlmClient` returns echo or canned responses.

**WorkflowConfig** (`config.rs`): Parsed from YAML with versioning, vars, and ordered steps.

**StepKind enum**: `ensure_dirs`, `write_file`, `split_sections`, `run_command`, `llm_generate`

### Data Flow

1. Parse YAML -> `WorkflowConfig`
2. `Runner::run()` iterates steps with timing
3. Each step executed via `execute_step()` in `steps.rs`
4. Template `{{var}}` substitution via `render.rs`
5. Produces `run.json` manifest with step reports

### Key Files

- `crates/vwf-core/src/engine.rs`: Runner and RunReport
- `crates/vwf-core/src/steps.rs`: Step implementations with payload deserialization
- `crates/vwf-core/src/runtime.rs`: Runtime/LlmClient traits and implementations
- `crates/vwf-core/src/render.rs`: `{{var}}` template substitution
- `crates/vwf-cli/src/main.rs`: CLI entry point with clap

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

## Python Package Management

**NEVER run `pip` or `pip3` directly.** Always use a virtual environment with `uv`:

```bash
# Create virtual environment
uv venv

# Activate the environment
source .venv/bin/activate

# Install packages using uv pip
uv pip install requests

# Or install from requirements
uv pip install -r requirements.txt
```

This ensures reproducible environments and avoids polluting the system Python installation.
