# Workflow Engine Concept: Reactive DAG Scheduler

## Core Idea

Instead of a linear sequence of steps, the workflow is a **Directed Acyclic Graph (DAG)** where:
- **Nodes** are tasks (generate text, render SVG, TTS, etc.)
- **Edges** are dependencies (TTS depends on script, video depends on TTS + image)
- **Scheduler** runs tasks when their dependencies are satisfied
- **Artifacts** are tracked with checksums for incremental rebuilds

This is similar to Make, Bazel, or Airflow but specialized for video production.

## Key Concepts

### 1. Artifacts (Files with Provenance)

```rust
struct Artifact {
    path: PathBuf,
    checksum: String,        // SHA256 of content
    created_at: DateTime,
    created_by: TaskId,
    inputs: Vec<ArtifactId>, // Dependencies that produced this
}
```

Artifacts are the "currency" of the system. Tasks consume and produce artifacts.

### 2. Tasks (Units of Work)

```rust
struct Task {
    id: TaskId,
    kind: TaskKind,
    inputs: Vec<InputSpec>,   // Required artifacts or values
    outputs: Vec<OutputSpec>, // Artifacts this will produce
    constraints: Constraints, // Sequential, GPU-bound, etc.
    status: TaskStatus,
}

enum TaskStatus {
    Blocked { waiting_on: Vec<TaskId> },
    Ready,
    Running,
    Complete { outputs: Vec<ArtifactId> },
    Failed { error: String },
    Skipped { reason: String },
}

struct Constraints {
    sequential_group: Option<String>, // e.g., "tts" - only one at a time
    resource: Option<String>,         // e.g., "gpu"
    parallelism: u32,                 // Max concurrent (default: unlimited)
}
```

### 3. Input Specifications

```rust
enum InputSpec {
    // Required artifact - task blocked until available
    Required { artifact: String },

    // Optional artifact - task runs with or without
    Optional { artifact: String, default: Option<Value> },

    // Placeholder - task runs with placeholder, re-runs when real arrives
    Placeholder { artifact: String, placeholder: PlaceholderKind },
}

enum PlaceholderKind {
    SolidColor { color: String },      // For missing images
    SilentAudio { duration: f64 },     // For missing TTS
    StaticImage { image: String },     // For missing video
    Skip,                               // Omit from output
}
```

### 4. Workflow Definition

```yaml
name: 5mlc-episode
version: 1

inputs:
  required:
    - episode_number: int
    - concepts: list[string]
  optional:
    - background_image: file
    - music_track: file
    - intro_video: file
    - outro_video: file

tasks:
  generate_outline:
    kind: llm_generate
    inputs:
      - episode_number
      - concepts
    outputs:
      - work/slides-outline.txt
      - work/narration.txt
      - work/description.txt

  checkpoint_review_text:
    kind: checkpoint
    inputs:
      - work/slides-outline.txt
      - work/narration.txt
    message: "Review text files before proceeding"

  create_slides:
    kind: html_template
    inputs:
      - work/slides-outline.txt
      - background_image: { placeholder: solid_color("#1a1a2e") }
    outputs:
      - showcase/slides/*.html

  capture_screenshots:
    kind: playwright_screenshot
    inputs:
      - showcase/slides/*.html
    outputs:
      - work/images/*.png
    constraints:
      parallelism: 4  # Can run 4 browsers in parallel

  generate_tts:
    kind: tts_generate
    inputs:
      - work/scripts/*.txt
    outputs:
      - work/audio/*.wav
    constraints:
      sequential_group: tts  # Only one TTS at a time
      resource: gpu

  # ... more tasks
```

### 5. Scheduler Algorithm

```
while incomplete_tasks exist:
    ready_tasks = tasks where all inputs available and not blocked

    for task in ready_tasks:
        if task.constraints.sequential_group:
            if another task in same group is running:
                continue  # Wait for it

        if task.constraints.resource:
            if resource is occupied:
                continue  # Wait for resource

        if parallelism_limit_reached:
            continue

        start_task(task)

    wait_for_any_task_completion()

    for completed_task:
        register_outputs(completed_task)
        check_dependent_tasks(completed_task)
        update_preview_if_possible()
```

### 6. Incremental Rebuilds

When an input changes (e.g., user provides background image):

1. Detect changed artifact (checksum differs)
2. Find all tasks that consumed this artifact
3. Invalidate their outputs
4. Find all tasks that consumed those outputs
5. Recursively invalidate
6. Re-run invalidated tasks (respecting constraints)

```rust
fn invalidate_downstream(artifact: &Artifact, state: &mut State) {
    for task in tasks_consuming(artifact) {
        for output in task.outputs {
            state.invalidate(output);
            invalidate_downstream(output, state);
        }
    }
}
```

### 7. Preview Generation

Special "preview" task that:
- Takes all video clips as inputs (required or placeholder)
- Produces work/preview.mp4
- Re-runs whenever any clip changes
- Uses placeholders for missing clips

```yaml
generate_preview:
    kind: concat_video
    inputs:
      - clips/00-intro.mp4: { placeholder: skip }
      - clips/01-*.mp4: { placeholder: static_image("placeholder.png") }
      - clips/99-outro.mp4: { placeholder: skip }
    outputs:
      - work/preview.mp4
    constraints:
      always_run: true  # Run after any clip changes
```

## State Persistence

```json
// work/state.json
{
  "workflow": "5mlc-episode",
  "started_at": "2026-02-15T10:00:00Z",
  "inputs": {
    "episode_number": 17,
    "concepts": ["attention", "transformer", ...],
    "background_image": null,
    "music_track": "reference/music/sly-sky.mp3"
  },
  "artifacts": {
    "work/narration.txt": {
      "checksum": "abc123...",
      "created_at": "2026-02-15T10:05:00Z",
      "created_by": "generate_outline"
    }
  },
  "tasks": {
    "generate_outline": { "status": "complete" },
    "checkpoint_review_text": { "status": "waiting" },
    "create_slides": { "status": "blocked", "waiting_on": ["checkpoint_review_text"] }
  },
  "checkpoints": {
    "checkpoint_review_text": { "approved": false }
  }
}
```

## Advantages Over Linear Workflow

| Linear | DAG-based |
|--------|-----------|
| Must provide all inputs upfront | Start with minimal inputs |
| Steps run in fixed order | Steps run when ready |
| One failure blocks everything | Parallel work continues |
| No incremental rebuilds | Only re-run affected tasks |
| No preview until complete | Preview at any stage |
| Manual restart after interrupt | Automatic resume |

## Implementation Phases

### Phase 1: Core Scheduler
- Task/Artifact/State structs
- Dependency resolution
- Sequential constraint enforcement
- State persistence

### Phase 2: Incremental Rebuilds
- Checksum tracking
- Change detection
- Downstream invalidation

### Phase 3: Placeholder System
- Placeholder generation
- Re-run when real arrives
- Preview assembly

### Phase 4: UI
- Web UI showing DAG state
- Real-time updates
- Checkpoint approval interface
