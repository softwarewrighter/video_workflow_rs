# Project Status

## Current State: Component Restructure Complete

The project has been restructured into three independent components (future git repositories) with clear dependency hierarchy.

## Last Updated: 2026-02-15

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | Complete | 100% |
| M2: Shell Step | Complete | 100% |
| M3: LLM Adapter | Partial | 30% (mock only) |
| M4: Web UI | Skeleton | 10% |
| M5: Component Restructure | Complete | 100% |

## Component Structure

```
components/
|-- vwf-foundation/   # 3 crates, 9 tests
|-- vwf-engine/       # 4 crates, 13 tests
|-- vwf-apps/         # 2 crates
```

### vwf-foundation (Layer 0)

| Crate | Status | Tests | Notes |
|-------|--------|-------|-------|
| vwf-types | Complete | 0 | TaskId, ArtifactId, TaskStatus, ArtifactStatus |
| vwf-runtime | Complete | 0 | Runtime trait, FsRuntime, DryRunRuntime, MockLlmClient |
| vwf-dag | Complete | 9 | Scheduler, Task, Artifact, State persistence |

### vwf-engine (Layer 1-4)

| Crate | Status | Tests | Notes |
|-------|--------|-------|-------|
| vwf-config | Complete | 5 | Workflow YAML parsing + validation |
| vwf-render | Complete | 2 | Template {{var}} substitution |
| vwf-steps | Complete | 6 | All 5 step types implemented |
| vwf-core | Complete | 0 | Engine orchestration, RunReport |

### vwf-apps (Layer 5)

| Crate | Status | Notes |
|-------|--------|-------|
| vwf-cli | Complete | Argument parsing, execution, dry-run |
| vwf-web | Skeleton | Yew UI, compiles but minimal functionality |

## Build System

```bash
# Build all components
./scripts/build-all.sh

# Test all components
./scripts/test-all.sh

# Build/test individual component
cd components/vwf-foundation && ./scripts/build-all.sh
```

## Test Coverage

| Component | Tests | Notes |
|-----------|-------|-------|
| vwf-foundation | 9 | DAG scheduler, state, task tests |
| vwf-engine | 13 | Config, render, steps tests |
| vwf-apps | 0 | Integration tests pending |
| **Total** | **22** | All passing |

## Recent Changes

- Restructured into 3 top-level components (future git submodules)
- Created vwf-types for shared type definitions
- Split vwf-core into vwf-config, vwf-render, vwf-steps, vwf-core
- Added build-all.sh and test-all.sh scripts at each level
- Clear dependency hierarchy: foundation -> engine -> apps
- No cyclic dependencies (enforced by crate structure)

## Dependency Graph

```
vwf-types (L0)
    |
    v
vwf-runtime + vwf-dag (L1)
    |
    v
vwf-config + vwf-render (L2)
    |
    v
vwf-steps (L3)
    |
    v
vwf-core (L4)
    |
    v
vwf-cli + vwf-web (L5)
```

## Known Issues

1. vwf-web path dependencies need verification for WASM builds
2. Integration tests not yet migrated to new structure

## Next Steps

1. Run sw-checklist on new component structure
2. Add integration tests to vwf-apps
3. Implement real LLM adapter in vwf-runtime

## Commands

```bash
# Build everything
./scripts/build-all.sh

# Test everything
./scripts/test-all.sh

# Run CLI
cargo run -p vwf-cli --manifest-path components/vwf-apps/Cargo.toml -- run examples/workflows/shorts_narration.yaml --workdir work/demo

# Dry run
cargo run -p vwf-cli --manifest-path components/vwf-apps/Cargo.toml -- run examples/workflows/shorts_narration.yaml --workdir work/demo --dry-run
```
