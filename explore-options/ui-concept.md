# UI Concept: Workflow State Visualization

## Overview

A web-based UI (Yew/WASM) that shows:
1. Current workflow state
2. Task dependencies (DAG visualization)
3. Progress and status of each task
4. Preview of completed artifacts
5. Checkpoint approval interface

## Main Dashboard

```
+------------------------------------------------------------------+
|  VWF Dashboard                                    [New] [Import] |
+------------------------------------------------------------------+
|                                                                  |
|  Active Projects                                                 |
|  +------------------------------------------------------------+  |
|  | 5MLC-17        | Shorts | 75% complete | [View] [Resume]   |  |
|  | ask-explainer  | Long   | 30% complete | [View] [Resume]   |  |
|  | proact-demo    | Both   | Pending      | [View] [Start]    |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Recent Outputs                                                  |
|  +------------------------------------------------------------+  |
|  | 5MLC-16 final.mp4      | 2:34 | Ready | [Play] [Upload]    |  |
|  | 5MLC-15 final.mp4      | 2:21 | Ready | [Play] [Uploaded]  |  |
|  +------------------------------------------------------------+  |
|                                                                  |
+------------------------------------------------------------------+
```

## Project View: Task DAG

```
+------------------------------------------------------------------+
|  5MLC-17: Five ML Concepts                        [Pause] [Stop] |
+------------------------------------------------------------------+
|                                                                  |
|  Progress: 9/14 tasks complete    Elapsed: 00:12:34              |
|  [=============================                    ] 64%         |
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                                                            |  |
|  |    [generate_outline]         [capture_screenshots]        |  |
|  |          |  DONE                    |  DONE                |  |
|  |          v                          v                      |  |
|  |    [CHECKPOINT]               [generate_tts]               |  |
|  |     REVIEW TEXT                 |  3/7 RUNNING             |  |
|  |     [Approve]                   v                          |  |
|  |          |                  [verify_tts]                   |  |
|  |          v                      |  WAITING                 |  |
|  |    [create_slides]              v                          |  |
|  |          |  DONE            [combine_audio]                |  |
|  |          v                      |  BLOCKED                 |  |
|  |    [split_scripts]              v                          |  |
|  |          |  DONE            [create_clips]                 |  |
|  |                                 |  BLOCKED                 |  |
|  |                                 v                          |  |
|  |                            [concat]                        |  |
|  |                                 |  BLOCKED                 |  |
|  |                                 v                          |  |
|  |                            [final.mp4]                     |  |
|  |                                                            |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Legend: [DONE] [RUNNING] [WAITING] [BLOCKED] [CHECKPOINT]       |
|                                                                  |
+------------------------------------------------------------------+
```

## Task Detail Panel

```
+------------------------------------------------------------------+
|  Task: generate_tts                                              |
+------------------------------------------------------------------+
|                                                                  |
|  Status: RUNNING (3/7 segments complete)                         |
|  Started: 2026-02-15 14:32:00                                    |
|  Elapsed: 00:04:23                                               |
|                                                                  |
|  Constraints:                                                    |
|    - Sequential (GPU-bound)                                      |
|    - Resource: gpu                                               |
|                                                                  |
|  Inputs:                                                         |
|    - work/scripts/00-intro.txt      [View]                       |
|    - work/scripts/01-concept-1.txt  [View]                       |
|    - ...                                                         |
|                                                                  |
|  Outputs:                                                        |
|    - work/audio/00-intro.wav        DONE   [Play]                |
|    - work/audio/01-concept-1.wav    DONE   [Play]                |
|    - work/audio/02-concept-2.wav    DONE   [Play]                |
|    - work/audio/03-concept-3.wav    RUNNING...                   |
|    - work/audio/04-concept-4.wav    PENDING                      |
|    - work/audio/05-concept-5.wav    PENDING                      |
|    - work/audio/06-cta.wav          PENDING                      |
|                                                                  |
|  Logs:                                                           |
|  +------------------------------------------------------------+  |
|  | [14:32:01] Starting TTS for 00-intro.txt                   |  |
|  | [14:32:15] Verifying with whisper... OK                    |  |
|  | [14:32:16] Starting TTS for 01-concept-1.txt               |  |
|  | ...                                                        |  |
|  +------------------------------------------------------------+  |
|                                                                  |
+------------------------------------------------------------------+
```

## Checkpoint Approval

```
+------------------------------------------------------------------+
|  CHECKPOINT: Review Text Content                                 |
+------------------------------------------------------------------+
|                                                                  |
|  Please review the following files before proceeding:            |
|                                                                  |
|  +------------------------------------------------------------+  |
|  | File                  | Status   | Actions                 |  |
|  |-----------------------|----------|-------------------------|  |
|  | work/narration.txt    | Ready    | [View] [Edit] [Diff]    |  |
|  | work/slides-outline   | Ready    | [View] [Edit] [Diff]    |  |
|  | work/description.txt  | Ready    | [View] [Edit] [Diff]    |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Preview:                                                        |
|  +------------------------------------------------------------+  |
|  | Slide 1: Attention                                         |  |
|  | "Attention mechanisms allow models to focus on relevant    |  |
|  |  parts of the input, like highlighting important words."   |  |
|  |                                                [Play TTS]  |  |
|  +------------------------------------------------------------+  |
|  | Slide 2: Transformer                                       |  |
|  | ...                                                        |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  [Approve & Continue]    [Request Changes]    [Regenerate All]   |
|                                                                  |
+------------------------------------------------------------------+
```

## Preview Player

```
+------------------------------------------------------------------+
|  Preview: 5MLC-17 (Current State)                                |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                                                            |  |
|  |                    [VIDEO PLAYER]                          |  |
|  |                                                            |  |
|  |  +------------------------------------------------------+  |  |
|  |  |                                                      |  |  |
|  |  |              Current preview frame                   |  |  |
|  |  |                                                      |  |  |
|  |  +------------------------------------------------------+  |  |
|  |                                                            |  |
|  |  [|<] [<] [PLAY/PAUSE] [>] [>|]     00:45 / 02:12          |  |
|  |                                                            |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Timeline:                                                       |
|  +------------------------------------------------------------+  |
|  | [SKIP] | 01-intro | 02-attn | 03-trans | ... | [SKIP]     |  |
|  |  intro |  REAL    |  REAL   | PLACEH.  |     |  outro     |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Missing Segments:                                               |
|    - 00-intro.mp4 (not provided)                                 |
|    - 03-transformer.mp4 (TTS pending)                            |
|    - 04-embedding.mp4 (TTS pending)                              |
|    - 99-outro.mp4 (not provided)                                 |
|                                                                  |
+------------------------------------------------------------------+
```

## Input Provider

```
+------------------------------------------------------------------+
|  Add Missing Inputs                                              |
+------------------------------------------------------------------+
|                                                                  |
|  Required (blocking):                                            |
|    None - all required inputs provided                           |
|                                                                  |
|  Optional (will improve output):                                 |
|  +------------------------------------------------------------+  |
|  | Input             | Status      | Actions                  |  |
|  |-------------------|-------------|--------------------------|  |
|  | background_image  | Not provided| [Upload] [Generate]      |  |
|  | intro_video       | Not provided| [Upload] [Skip]          |  |
|  | outro_video       | Not provided| [Upload] [Skip]          |  |
|  | music_track       | Default     | [Change] [Keep Default]  |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  Drop files here or click to upload                              |
|  +------------------------------------------------------------+  |
|  |                                                            |  |
|  |                  [ Drop Zone ]                             |  |
|  |                                                            |  |
|  +------------------------------------------------------------+  |
|                                                                  |
|  When new inputs are added, affected tasks will automatically    |
|  re-run and the preview will update.                             |
|                                                                  |
+------------------------------------------------------------------+
```

## Resource Monitor

```
+------------------------------------------------------------------+
|  Resources                                                       |
+------------------------------------------------------------------+
|                                                                  |
|  GPU (curiosity):                                                |
|  [====================                    ] 50% - TTS running    |
|                                                                  |
|  CPU:                                                            |
|  [========                                ] 20% - 2 ffmpeg jobs  |
|                                                                  |
|  Queue:                                                          |
|  +------------------------------------------------------------+  |
|  | Task              | Resource | Position | Est. Start       |  |
|  |-------------------|----------|----------|------------------|  |
|  | tts_04_embedding  | GPU      | 1        | ~2 min           |  |
|  | tts_05_loss       | GPU      | 2        | ~4 min           |  |
|  | tts_06_cta        | GPU      | 3        | ~6 min           |  |
|  +------------------------------------------------------------+  |
|                                                                  |
+------------------------------------------------------------------+
```

## Implementation Notes

### Technology Stack
- **Frontend:** Yew (Rust/WASM)
- **State:** Shared state via WebSocket to backend
- **Backend:** Axum server running workflow engine
- **Updates:** Server-sent events for real-time status

### Key Components

```rust
// Frontend state
struct AppState {
    projects: Vec<Project>,
    active_project: Option<ProjectId>,
    tasks: HashMap<TaskId, TaskState>,
    artifacts: HashMap<ArtifactId, Artifact>,
    checkpoints: HashMap<CheckpointId, CheckpointState>,
}

// WebSocket messages
enum WsMessage {
    TaskStarted { task_id: TaskId },
    TaskProgress { task_id: TaskId, progress: f32 },
    TaskComplete { task_id: TaskId, outputs: Vec<ArtifactId> },
    TaskFailed { task_id: TaskId, error: String },
    CheckpointReady { checkpoint_id: CheckpointId },
    PreviewUpdated { preview_path: String },
    InputProvided { input_name: String, path: String },
}
```

### File Watching

Backend watches for new files in expected locations:

```rust
// Watch for user-provided inputs
watch("assets/") -> on_new_file -> register_input -> invalidate_downstream

// Watch for edited scripts (user made changes during checkpoint)
watch("work/scripts/") -> on_change -> invalidate_tts
```
