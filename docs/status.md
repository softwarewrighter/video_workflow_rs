# Project Status

## Current State: Skeleton Implementation

The project has a working skeleton with core modules defined but incomplete implementations.

## Last Updated: 2026-02-15

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | In Progress | 40% |
| M2: Shell Step | Not Started | 0% |
| M3: LLM Adapter | Not Started | 0% |
| M4: Web UI | Not Started | 0% |

## Component Status

### vwf-core (Engine)

| Component | Status | Notes |
|-----------|--------|-------|
| config.rs | Partial | Basic structs, needs validation |
| engine.rs | Partial | Runner skeleton, needs manifest |
| render.rs | Partial | Basic substitution, needs error handling |
| runtime.rs | Partial | Traits defined, implementations incomplete |
| steps.rs | Partial | ensure_dirs and write_file started |

### vwf-cli

| Component | Status | Notes |
|-----------|--------|-------|
| Argument parsing | Basic | Needs --dry-run, --var, --allow |
| Workflow execution | Partial | Calls core, no manifest output |
| Error reporting | Basic | Needs step ID context |

### vwf-web

| Component | Status | Notes |
|-----------|--------|-------|
| Yew setup | Skeleton | main.rs exists, no UI |

### Examples

| Workflow | Status | Notes |
|----------|--------|-------|
| shorts_narration.yaml | Complete | Demonstrates all step types |
| explainer_outline.yaml | Exists | Needs verification |

## Test Coverage

| Module | Unit Tests | Integration | Notes |
|--------|------------|-------------|-------|
| config | None | None | Needs parsing tests |
| engine | None | None | Needs runner tests |
| render | None | None | Needs template tests |
| runtime | None | None | Needs mock tests |
| steps | None | None | Needs step tests |

## Known Issues

1. **No validation errors with step IDs**: Errors don't include context
2. **split_sections not implemented**: Step exists in YAML but code incomplete
3. **No run.json generation**: Manifest output not implemented
4. **No dry-run mode**: CLI flag not wired up
5. **No integration tests**: Missing temp directory tests

## Recent Changes

- Initial skeleton from ChatGPT zip
- Workspace structure with three crates
- Example workflows defined
- Documentation created

## Blockers

None currently. Ready for TDD implementation.

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
