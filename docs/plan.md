# Implementation Plan

## Current Phase: Milestone 1 (M1) - Workflow Runner

The skeleton is in place. Focus is on making the CLI run YAML workflows deterministically.

## Task Breakdown

### Phase 1: Core Engine (M1)

#### 1.1 YAML Parsing [In Progress]
- [x] Define WorkflowConfig struct
- [x] Define StepConfig enum with serde tagging
- [ ] Add comprehensive validation
- [ ] Error messages include step ID and remediation

#### 1.2 Template Rendering [In Progress]
- [x] Basic `{{var}}` substitution
- [ ] Error on missing variables
- [ ] Handle edge cases (empty vars, nested braces)

#### 1.3 Step Implementations [In Progress]
- [x] ensure_dirs step
- [x] write_file step
- [ ] split_sections step
- [ ] Robust error handling per step

#### 1.4 Engine Runner [In Progress]
- [x] Sequential step execution
- [ ] Timing capture per step
- [ ] Status tracking (Ok/Skipped/Failed)
- [ ] Manifest generation (run.json)

#### 1.5 CLI Integration [In Progress]
- [x] Basic argument parsing
- [ ] --dry-run flag
- [ ] --var key=value overrides
- [ ] Proper exit codes

### Phase 2: Command Execution (M2)

#### 2.1 run_command Step
- [ ] Command parsing and validation
- [ ] Allowlist enforcement
- [ ] Working directory handling
- [ ] Stdout/stderr capture

#### 2.2 Safety Features
- [ ] --allow <cmd> CLI flag
- [ ] Error with remediation on unknown command
- [ ] Artifact storage for output

### Phase 3: LLM Integration (M3)

#### 3.1 LlmClient Trait
- [x] Define trait interface
- [x] MockLlmClient for tests
- [ ] Request/response structs

#### 3.2 Claude Code CLI Adapter
- [ ] Binary path configuration
- [ ] Argument construction
- [ ] Output parsing
- [ ] Feature flag gating

### Phase 4: Web UI (M4)

#### 4.1 Yew Skeleton
- [ ] Workflow list view
- [ ] Variable form (mad-lib)
- [ ] File upload
- [ ] RunRequest JSON generation

#### 4.2 HTTP Runner
- [ ] Local server endpoint
- [ ] Request validation
- [ ] Async execution
- [ ] Status polling

## TDD Test Sequence

Follow this order for red/green testing:

1. **Parse minimal workflow YAML** into structs
2. **Validate unknown step type** returns error with step ID
3. **Render templates** with `{{var}}` substitution
4. **Render templates** errors on missing var
5. **Write artifact** step creates file
6. **Manifest references** created artifacts
7. **Dry run** produces no filesystem mutations
8. **Full run** writes manifest + artifacts
9. **Command allowlist** rejects unknown commands
10. **Capture logs** stores stdout/stderr
11. **LLM mock** returns configured response

## File Checklist

### Must Exist
- [x] Cargo.toml (workspace)
- [x] crates/vwf-core/Cargo.toml
- [x] crates/vwf-core/src/lib.rs
- [x] crates/vwf-core/src/config.rs
- [x] crates/vwf-core/src/engine.rs
- [x] crates/vwf-core/src/render.rs
- [x] crates/vwf-core/src/runtime.rs
- [x] crates/vwf-core/src/steps.rs
- [x] crates/vwf-cli/Cargo.toml
- [x] crates/vwf-cli/src/main.rs
- [x] examples/workflows/shorts_narration.yaml

### To Create
- [ ] crates/vwf-core/src/manifest.rs (run.json schema)
- [ ] crates/vwf-core/src/validation.rs (if split from config)
- [ ] crates/vwf-cli/src/commands/run.rs (if modular CLI)

## Development Workflow

```
1. Pick next test from sequence
2. Write failing test (RED)
3. Implement minimal code (GREEN)
4. Refactor if needed
5. Run full test suite
6. Commit with descriptive message
7. Repeat
```

## Dependencies

### Required
- serde, serde_yaml: YAML parsing
- thiserror, anyhow: Error handling
- clap: CLI argument parsing
- chrono: Timestamps
- tracing: Logging

### Optional (Future)
- tokio: Async runtime (for HTTP runner)
- axum: HTTP framework (for runner service)
- yew: Web UI framework
- wasm-bindgen: WASM bindings

## Blockers and Risks

| Risk | Mitigation |
|------|------------|
| YAML schema changes | Version field for compatibility |
| LLM API changes | Trait abstraction isolates core |
| Command injection | Strict allowlist enforcement |
| Large file handling | Stream if needed (V2) |

## Next Actions

1. Complete split_sections step implementation
2. Add comprehensive validation error messages
3. Implement run.json manifest generation
4. Add --dry-run to CLI
5. Write integration tests with temp directories
