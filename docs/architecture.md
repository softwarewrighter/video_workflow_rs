# Architecture

## Overview

VWF (Video Workflow Framework) is a Rust-based workflow engine designed to eliminate "agent drift" in video production pipelines. The core principle is **workflows are data, runner is code** - all configuration is explicit YAML, and execution is deterministic.

```
+-------------------+     +------------------+     +-------------------+
|   Workflow YAML   | --> |   vwf-engine     | --> |   Artifacts +     |
|   + Variables     |     |   (Core)         |     |   run.json        |
+-------------------+     +------------------+     +-------------------+
        ^                         |
        |                         v
+-------------------+     +------------------+
|   vwf-apps        |     |   vwf-foundation |
|   (CLI + Web)     |     |   (Runtime+DAG)  |
+-------------------+     +------------------+
```

## Component Structure

The project is organized into three top-level components, each designed to become a separate git repository:

```
video_workflow_rs/
|-- components/
|   |-- vwf-foundation/        # Layer 0: External deps only
|   |   |-- Cargo.toml         # Workspace root
|   |   |-- crates/
|   |   |   |-- vwf-types/     # Shared type definitions
|   |   |   |-- vwf-runtime/   # Runtime abstraction layer
|   |   |   |-- vwf-dag/       # DAG scheduler primitives
|   |   |-- scripts/
|   |       |-- build-all.sh
|   |       |-- test-all.sh
|   |
|   |-- vwf-engine/            # Layer 1: Depends on foundation
|   |   |-- Cargo.toml         # Workspace root
|   |   |-- crates/
|   |   |   |-- vwf-config/    # Workflow configuration
|   |   |   |-- vwf-render/    # Template rendering
|   |   |   |-- vwf-steps/     # Step implementations
|   |   |   |-- vwf-core/      # Engine orchestration
|   |   |-- scripts/
|   |       |-- build-all.sh
|   |       |-- test-all.sh
|   |
|   |-- vwf-apps/              # Layer 2: Depends on engine
|       |-- Cargo.toml         # Workspace root
|       |-- crates/
|       |   |-- vwf-cli/       # Command-line interface
|       |   |-- vwf-web/       # Yew/WASM UI
|       |-- scripts/
|           |-- build-all.sh
|           |-- test-all.sh
|
|-- scripts/
|   |-- build-all.sh           # Uber build script
|   |-- test-all.sh            # Uber test script
|
|-- examples/
|-- docs/
```

## Dependency Hierarchy (No Cycles)

```
Layer 0: vwf-types (external crates only)
         |
         v
Layer 1: vwf-runtime, vwf-dag (depend on vwf-types)
         |
         v
Layer 2: vwf-config, vwf-render (depend on Layer 1)
         |
         v
Layer 3: vwf-steps (depends on Layer 2)
         |
         v
Layer 4: vwf-core (depends on Layer 3)
         |
         v
Layer 5: vwf-cli, vwf-web (depend on Layer 4)
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

## Key Crates

### vwf-foundation

- **vwf-types**: Core type definitions (TaskId, ArtifactId, TaskStatus, ArtifactStatus)
- **vwf-runtime**: Runtime trait and implementations (FsRuntime, DryRunRuntime, MockLlmClient)
- **vwf-dag**: DAG scheduler, task/artifact tracking, state persistence

### vwf-engine

- **vwf-config**: Workflow YAML parsing and validation
- **vwf-render**: Template variable substitution
- **vwf-steps**: Step handler implementations
- **vwf-core**: Engine orchestration and reporting

### vwf-apps

- **vwf-cli**: Command-line interface
- **vwf-web**: Yew/WASM browser UI

## Build System

Each component has its own workspace and build scripts. The uber scripts coordinate builds in dependency order:

```bash
# Build everything
./scripts/build-all.sh

# Test everything
./scripts/test-all.sh
```

## Security Considerations

- **Command Allowlist**: `run_command` requires explicit `--allow` flags
- **No Network in Tests**: All LLM calls go through mockable trait
- **Explicit Paths**: No implicit current directory; workdir is always specified

## Extension Points

1. **New Step Types**: Add variant to `StepKind` enum, implement in vwf-steps
2. **LLM Providers**: Implement `LlmClient` trait (mock, Claude Code CLI, API)
3. **Output Formats**: Extend `RunReport` for different consumers

## Non-Functional Requirements

- **Determinism**: Stable paths, no random names unless explicit
- **Auditability**: Full provenance in run.json
- **Testability**: Runtime trait enables pure unit tests
- **Explainable Failures**: Errors include what, where, and how to fix
