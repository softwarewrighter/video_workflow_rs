# Project Status

## Current State: Sample Video Phase 1 Complete

All components pass sw-checklist (0 failures). Phase 1 of sample video production is complete - scripts generated successfully.

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
| M7: Sample Video Phase 1 | Complete | 100% |
| M8: Sample Video Phase 2 | Complete | 100% |

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

## Sample Video Project

**Location:** `test-projects/sample-video/`

**Generated Artifacts:**
- `work/scripts/*.txt` - 5 narration script sections
- `work/audio/*.wav` - 5 voice-cloned audio files (~30 sec total)
- `output/manifest.txt` - Production summary

**Audio Durations (VoxCPM voice clone):**
- 01-hook.wav: 4.2 sec
- 02-problem.wav: 5.8 sec
- 03-solution.wav: 6.7 sec
- 04-benefit.wav: 7.0 sec
- 05-cta.wav: 5.8 sec

## Recent Changes

- **Phase 2 Complete:** Voice-cloned audio with VoxCPM (~30 sec total)
- **Phase 1 Complete:** Sample video scripts generated via workflow
- Fixed llm_generate bug that appended user prompt to mock response
- All components pass sw-checklist (0 failures)
- Added footer to vwf-web with copyright/license info
- Reduced function counts across all crates to meet 7-function limit
- Split vwf-cli and vwf-web into smaller modules
- Consolidated vwf-dag scheduler and state modules
- Restructured into 3 top-level components (future git submodules)

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

### Phase 1: Script Generation ✅ COMPLETE
- Created `test-projects/sample-video/` with workflow.yaml
- Generated 5 script sections via mock LLM + split_sections
- Output: work/scripts/{01-hook,02-problem,03-solution,04-benefit,05-cta}.txt

### Phase 2: TTS Integration ✅ COMPLETE
- Using VoxCPM voice cloning via Gradio client (curiosity:7860)
- Reference: mike-medium-ref-1.wav (63s) with matching transcript
- Generated voice-cloned audio for all 5 script sections
- Output: work/audio/*.wav (~30 seconds total)

### Phase 3: Real LLM Integration
1. Implement Claude API adapter in vwf-runtime
2. Replace mock LLM with real generation
3. Test full workflow with actual AI-generated content

### Phase 4: Video Assembly
1. Add ffmpeg steps for image+audio composition
2. Concatenate segments into final video
3. Add intro/outro, transitions, background music

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
