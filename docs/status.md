# Project Status

## Current State: M1 Complete

The workflow runner is fully functional with validation, dry-run support, and comprehensive tests.

## Last Updated: 2026-02-15

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | Complete | 100% |
| M2: Shell Step | Complete | 100% |
| M3: LLM Adapter | Partial | 30% (mock only) |
| M4: Web UI | Not Started | 0% |

## Component Status

### vwf-core (Engine)

| Component | Status | Notes |
|-----------|--------|-------|
| config.rs | Complete | Parsing + validation (duplicate IDs, empty IDs) |
| engine.rs | Complete | Runner with timing, manifest generation |
| render.rs | Complete | {{var}} substitution with error on missing |
| runtime.rs | Complete | FsRuntime, DryRunRuntime (can read planned writes) |
| steps.rs | Complete | All 5 step types implemented |

### vwf-cli

| Component | Status | Notes |
|-----------|--------|-------|
| Argument parsing | Complete | --dry-run, --var, --allow all work |
| Workflow execution | Complete | Produces run.json manifest |
| Error reporting | Complete | Includes step ID in all errors |

### vwf-web

| Component | Status | Notes |
|-----------|--------|-------|
| Yew setup | Skeleton | main.rs exists, compiles, no UI logic |

### Examples

| Workflow | Status | Notes |
|----------|--------|-------|
| shorts_narration.yaml | Verified | Full dry-run and real execution tested |
| explainer_outline.yaml | Exists | Not yet verified |

## Test Coverage

| Module | Unit Tests | Integration | Notes |
|--------|------------|-------------|-------|
| config | 5 | - | Parsing, validation, unknown kinds |
| engine | 1 | - | Basic runner test |
| render | 2 | - | Substitution, missing var |
| runtime | - | - | Tested via integration |
| steps | 1 | - | split_sections |
| lib.rs | - | 10 | Full workflow tests |

**Total: 19 tests passing**

## Known Issues

1. **Mock LLM echo includes user prompt**: When using mock_response, the echoed output includes both mock response and original prompt. Real LLM adapter will fix this.

## Recent Changes

- Completed M1: Workflow runner with all step types
- Completed M2: Shell step with allowlist enforcement
- Added WorkflowConfig::from_yaml() with validation
- DryRunRuntime can read files written during same run
- 19 comprehensive tests added
- CLI produces run.json manifest on every run

## Blockers

None currently.

## Next Steps (Priority Order)

1. **Write first failing test**: Parse minimal workflow YAML
2. **Implement validation**: Unknown step type returns error with ID
3. **Complete template rendering**: Error on missing variables
4. **Implement split_sections**: Extract sections by heading
5. **Add manifest generation**: Write run.json on completion
6. **Wire up --dry-run**: No filesystem mutations in dry mode

## Commands

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run example workflow (once implemented)
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo

# Dry run (once implemented)
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo --dry-run
```

## Development Notes

### TDD Process

Follow red/green cycle strictly:
1. Write failing test
2. Implement minimal code to pass
3. Refactor only when green
4. Commit after each cycle

### Quality Gates

Before each commit:
- `cargo test` - All tests pass
- `cargo clippy -- -D warnings` - No warnings
- `cargo fmt` - Code formatted

### Documentation

Keep updated:
- docs/status.md - Current state
- docs/plan.md - Task progress
- docs/architecture.md - System changes
