# Project Status

## Current State: Ready for Sample Video Production

All components pass sw-checklist (0 failures). The framework is ready to produce a sample video to validate the end-to-end workflow.

## Last Updated: 2026-02-15

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | Complete | 100% |
| M2: Shell Step | Complete | 100% |
| M3: LLM Adapter | Partial | 30% (mock only) |
| M4: Web UI | Skeleton | 15% (footer added) |
| M5: Component Restructure | Complete | 100% |
| M6: sw-checklist Compliance | Complete | 100% |

## Component Structure

```
components/
|-- vwf-foundation/   # 3 crates, 9 tests
|-- vwf-engine/       # 4 crates, 11 tests
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
| vwf-engine | 11 | Config, render, steps tests |
| vwf-apps | 0 | Integration tests pending |
| **Total** | **20** | All passing |

## Recent Changes

- All components pass sw-checklist (0 failures, warnings only)
- Added footer to vwf-web with copyright/license info
- Reduced function counts across all crates to meet 7-function limit
- Split vwf-cli and vwf-web into smaller modules
- Consolidated vwf-dag scheduler and state modules
- Restructured into 3 top-level components (future git submodules)
- Clear dependency hierarchy: foundation -> engine -> apps

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
2. Real LLM adapter not yet implemented (using mock)

## Next Steps: Sample Video Production

### Goal
Produce a complete sample video using the VWF framework to validate the end-to-end workflow.

### Phase 1: Prepare Assets
1. Create a sample project in `test-projects/sample-video/`
2. Write a simple script topic (30-60 seconds of content)
3. Gather placeholder assets (images, background music)

### Phase 2: Workflow Execution
1. Create workflow YAML for the sample video
2. Run with mock LLM to generate narration script
3. Integrate TTS step (e.g., edge-tts or similar)
4. Add ffmpeg steps for video assembly

### Phase 3: Real LLM Integration
1. Implement Claude API adapter in vwf-runtime
2. Replace mock LLM with real generation
3. Test full workflow with actual AI-generated content

### Phase 4: Polish
1. Add integration tests based on sample video workflow
2. Document the sample video production process
3. Create reusable workflow templates

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
