# Architecture

## Overview

VWF (Video Workflow Framework) is a Rust-based workflow engine designed to eliminate "agent drift" in video production pipelines. The core principle is **workflows are data, runner is code** - all configuration is explicit YAML, and execution is deterministic.

```
+-------------------+     +------------------+     +-------------------+
|   Workflow YAML   | --> |   vwf-core       | --> |   Artifacts +     |
|   + Variables     |     |   (Engine)       |     |   run.json        |
+-------------------+     +------------------+     +-------------------+
        ^                         |
        |                         v
+-------------------+     +------------------+
|   vwf-cli         |     |   Runtime        |
|   (User Interface)|     |   (Side Effects) |
+-------------------+     +------------------+
        ^
        |
+-------------------+
|   vwf-web (Yew)   |
|   (Future UI)     |
+-------------------+
```

## Workspace Structure

```
video_workflow_rs/
|-- Cargo.toml              # Workspace root
|-- crates/
|   |-- vwf-core/           # Engine + steps + runtime traits
|   |-- vwf-cli/            # Command-line interface
|   |-- vwf-web/            # Yew/WASM UI (future)
|-- examples/
|   |-- workflows/          # Example YAML workflows
|-- docs/                   # Documentation
|-- work/                   # Runtime artifacts (gitignored)
```

## Core Concepts

### 1. WorkflowConfig (YAML)

Versioned workflow definition with:
- **version**: Schema version for forward compatibility
- **name/description**: Human-readable metadata
- **vars**: Key-value template variables
- **steps**: Ordered list of step configurations

### 2. Runner (Engine)

Executes steps sequentially:
- Resolves `{{var}}` templates against context
- Captures timing and status for each step
- Produces `run.json` manifest with provenance

### 3. Runtime (Side Effects Boundary)

All I/O goes through the `Runtime` trait:
- **FsRuntime**: Real filesystem + command execution
- **DryRunRuntime**: Preview without mutations
- **LlmClient trait**: Mockable LLM adapter

This enables unit tests with no network/disk I/O.

### 4. Steps (Explicit Units)

Each step has:
- **id**: Unique identifier (referenced in errors)
- **kind**: Step type (ensure_dirs, write_file, llm_generate, etc.)
- **inputs/outputs**: Explicit paths, no magic defaults

## Data Flow

```
1. Parse YAML --> WorkflowConfig
2. Validate --> Fail fast with step id + field + remediation
3. For each step:
   a. Render templates ({{var}} substitution)
   b. Execute via Runtime
   c. Record status, timing, artifacts
4. Write run.json manifest
```

## Key Modules

### vwf-core/config.rs
- `WorkflowConfig`: Deserialize from YAML
- `StepConfig`: Enum over step types
- Validation logic with actionable errors

### vwf-core/engine.rs
- `Runner`: Orchestrates step execution
- `RunReport`: Manifest with step statuses
- `StepStatus`: Ok, Skipped, Failed(reason)

### vwf-core/runtime.rs
- `Runtime` trait: Abstract filesystem + commands
- `FsRuntime`: Production implementation
- `DryRunRuntime`: No-op for preview
- `LlmClient` trait: Mock or real LLM calls

### vwf-core/render.rs
- `{{var}}` template substitution
- Error on missing variables (no silent defaults)

### vwf-core/steps.rs
- Step implementations: ensure_dirs, write_file, split_sections, run_command, llm_generate

## Security Considerations

- **Command Allowlist**: `run_command` requires explicit `--allow` flags
- **No Network in Tests**: All LLM calls go through mockable trait
- **Explicit Paths**: No implicit current directory; workdir is always specified

## Extension Points

1. **New Step Types**: Add variant to `StepConfig` enum, implement in steps.rs
2. **LLM Providers**: Implement `LlmClient` trait (mock, Claude Code CLI, API)
3. **Output Formats**: Extend `RunReport` for different consumers

## Non-Functional Requirements

- **Determinism**: Stable paths, no random names unless explicit
- **Auditability**: Full provenance in run.json
- **Testability**: Runtime trait enables pure unit tests
- **Explainable Failures**: Errors include what, where, and how to fix
