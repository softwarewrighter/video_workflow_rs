# Implementation Roadmap

## Current State

VWF has a working **linear workflow engine** (M1/M2 complete):
- YAML workflow parsing and validation
- Sequential step execution
- Template substitution
- Dry-run mode
- Command allowlist
- run.json manifest

## Target State

A **reactive DAG-based workflow engine** with:
- Dependency-driven execution
- Parallel execution where safe
- Sequential constraints (GPU-bound tasks)
- Progressive refinement (placeholders for missing inputs)
- Incremental rebuilds (only re-run affected tasks)
- Real-time preview generation
- Web UI for state visualization

## Migration Path

### Phase 1: Core DAG Infrastructure

**Goal:** Replace linear execution with DAG-based scheduling

**Tasks:**
1. Define `Task` struct with inputs/outputs/constraints
2. Define `Artifact` struct with checksum tracking
3. Implement dependency resolution (topological sort)
4. Implement scheduler with constraint enforcement
5. Persist state to `work/state.json`

**Deliverables:**
- `crates/vwf-core/src/dag.rs` - DAG data structures
- `crates/vwf-core/src/scheduler.rs` - Task scheduling
- `crates/vwf-core/src/artifacts.rs` - Artifact tracking

**Tests:**
- DAG with no dependencies runs all tasks
- DAG with chain runs in order
- DAG with parallel branches runs concurrently
- Sequential constraint prevents parallel execution
- Task failure stops dependents

### Phase 2: Incremental Rebuilds

**Goal:** Only re-run tasks when inputs change

**Tasks:**
1. Implement checksum calculation for artifacts
2. Track input checksums in state.json
3. Detect changed inputs on resume
4. Invalidate downstream tasks
5. Re-run only invalidated tasks

**Deliverables:**
- `crates/vwf-core/src/checksums.rs` - SHA256 tracking
- Enhanced `state.json` with checksums
- `vwf resume` command

**Tests:**
- Unchanged inputs skip re-execution
- Changed input invalidates downstream
- New input triggers affected tasks only

### Phase 3: Placeholder System

**Goal:** Run tasks with placeholders for missing optional inputs

**Tasks:**
1. Define `InputSpec::Optional` and `InputSpec::Placeholder`
2. Generate placeholder artifacts (solid color, silent audio)
3. Track placeholder vs. real artifacts
4. Re-run when real artifact replaces placeholder
5. Generate preview with placeholders

**Deliverables:**
- Placeholder generators in `crates/vwf-core/src/placeholders.rs`
- Preview assembly task
- `vwf preview` command

**Tests:**
- Missing optional input uses placeholder
- Task runs with placeholder
- Real input triggers re-run
- Preview includes available + placeholder

### Phase 4: Checkpoint System

**Goal:** Pause workflow for user review, resume on approval

**Tasks:**
1. Define `Checkpoint` task type
2. Pause scheduler at checkpoint
3. Persist checkpoint state
4. `vwf approve <checkpoint>` command
5. `vwf reject <checkpoint>` command (triggers regeneration)

**Deliverables:**
- Checkpoint handling in scheduler
- CLI commands for approval
- State persistence for checkpoints

**Tests:**
- Checkpoint pauses execution
- Approve continues execution
- Reject triggers regeneration
- Resume after interrupt respects checkpoint

### Phase 5: Video-Specific Tasks

**Goal:** Implement all task types needed for video production

**Tasks:**
1. `tts_generate` - VoxCPM via Python client
2. `tts_verify` - Whisper verification
3. `screenshot` - playwright-cli integration
4. `svg_render` - rsvg-convert wrapper
5. `ffmpeg` - Typed ffmpeg operations
6. `vid_concat` - vid-concat wrapper
7. `vid_normalize` - vid-volume wrapper
8. `llm_generate` - Cloud LLM for text generation

**Deliverables:**
- Task implementations in `crates/vwf-core/src/tasks/`
- Configuration for external tools (paths, servers)

**Tests:**
- Each task type with mock/real execution
- Error handling for each task type

### Phase 6: Workflow Templates

**Goal:** Define reusable workflow templates for Shorts and Explainers

**Tasks:**
1. Template YAML schema
2. Template instantiation with variables
3. 5MLC Shorts template
4. Explainer template
5. Dual-output template

**Deliverables:**
- `templates/5mlc-shorts.yaml`
- `templates/explainer.yaml`
- `templates/dual.yaml`
- `vwf new <template> --var ...` command

**Tests:**
- Template parsing and validation
- Variable substitution
- Full workflow from template

### Phase 7: Web UI

**Goal:** Visualize workflow state and provide checkpoint approval

**Tasks:**
1. Axum backend with WebSocket
2. State broadcasting via SSE/WebSocket
3. Yew frontend with DAG visualization
4. Task detail panel
5. Checkpoint approval interface
6. Preview player
7. Input upload

**Deliverables:**
- `crates/vwf-server/` - Axum backend
- `crates/vwf-web/` - Yew frontend (enhanced)
- Real-time updates

**Tests:**
- WebSocket connection
- State synchronization
- UI interactions

### Phase 8: File Watching

**Goal:** Automatically detect new inputs and trigger rebuilds

**Tasks:**
1. Watch `assets/` for new files
2. Watch `work/scripts/` for edits during checkpoint
3. Trigger invalidation on change
4. Update UI on change

**Deliverables:**
- File watcher integration (notify crate)
- Auto-invalidation

**Tests:**
- New file triggers task
- Edit triggers re-run
- Delete is handled gracefully

## Timeline Estimate

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| 1. Core DAG | 2-3 days | None |
| 2. Incremental | 1-2 days | Phase 1 |
| 3. Placeholders | 1-2 days | Phase 1 |
| 4. Checkpoints | 1 day | Phase 1 |
| 5. Video Tasks | 3-4 days | Phase 1 |
| 6. Templates | 1-2 days | Phase 5 |
| 7. Web UI | 3-5 days | Phase 1-4 |
| 8. File Watching | 1 day | Phase 2 |

**Total: ~2-3 weeks**

## Recommended Order

1. **Phase 1** (Core DAG) - Foundation for everything
2. **Phase 5** (Video Tasks) - Can test with linear execution first
3. **Phase 4** (Checkpoints) - Critical for workflow usability
4. **Phase 3** (Placeholders) - Enables progressive refinement
5. **Phase 2** (Incremental) - Optimization
6. **Phase 6** (Templates) - User-facing
7. **Phase 7** (Web UI) - Nice to have
8. **Phase 8** (File Watching) - Polish

## Parallel Development

While implementing the engine:
- Create concrete workflow YAMLs from refactor docs
- Test with real video production
- Iterate on task definitions
- Refine placeholder strategies

## Migration Strategy

Keep existing linear engine working during development:
- New code in `dag`, `scheduler`, `artifacts` modules
- Feature flag for DAG mode: `--engine dag`
- Default to linear until DAG is stable
- Gradual migration of workflows

```bash
# Linear (current)
vwf run workflow.yaml --workdir work

# DAG (new)
vwf run workflow.yaml --workdir work --engine dag

# Eventually make DAG the default
```
