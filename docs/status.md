# Project Status

## Current State: Video Pipeline Complete

All components pass sw-checklist (0 failures). Full video pipeline working: script generation, TTS voice cloning, video assembly.

## Last Updated: 2026-02-16

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | Complete | 100% |
| M2: Shell Step | Complete | 100% |
| M3: LLM Adapter | Partial | 30% (mock only) |
| M4: Web UI | Skeleton | 15% (footer added) |
| M5: Component Restructure | Complete | 100% |
| M6: sw-checklist Compliance | Complete | 100% |
| M7: Sample Video Pipeline | Complete | 100% |
| M8: GPU Queue + Resume | Complete | 100% |

## Quick Start

```bash
# Generate a sample short (~32 sec)
./scripts/demo-short.sh

# Generate a sample explainer (longer format)
./scripts/demo-explainer.sh
```

## New Features

### GPU Queue (vwf-queue)

Semaphore-based queue for serializing GPU-bound tasks:

```rust
let queue = GpuQueue::new(1, 2);  // TTS=1, Lipsync=2
let result = queue.run_tts(async { ... }).await;
```

### TTS Step (tts_generate)

Voice cloning via VoxCPM Gradio API:

```yaml
- id: tts_narration
  kind: tts_generate
  output_path: work/audio/narration.wav
  script_path: work/scripts/narration.txt
  reference_audio: /path/to/reference.wav
  reference_text: "Transcript of reference audio..."
  server: http://curiosity:7860
```

### Resume Support (--resume)

Skip steps whose outputs already exist:

```bash
vwf run workflow.yaml --workdir work --resume
```

Steps declare `output_path` for completion checking. Media files validated via ffprobe duration.

## Component Structure

```
components/
|-- vwf-foundation/   # 4 crates, 10 tests
|   |-- vwf-types
|   |-- vwf-runtime   # + output_is_valid()
|   |-- vwf-dag
|   +-- vwf-queue     # NEW: GPU semaphores
|-- vwf-engine/       # 4 crates, 12 tests
|   |-- vwf-config    # + output_path, TtsGenerate
|   |-- vwf-render
|   |-- vwf-steps     # + tts_generate
|   +-- vwf-core      # + RunOptions, resume
+-- vwf-apps/         # 2 crates
    |-- vwf-cli       # + --resume flag
    +-- vwf-web
```

## Sample Video Project

**Location:** `test-projects/sample-video/`

**Demo Scripts:**
- `scripts/generate-tts.sh` - Generate voice-cloned audio
- `scripts/verify-tts.sh` - Verify with Whisper
- `scripts/build-video.sh` - Assemble final video

**Generated Artifacts:**
- `work/scripts/*.txt` - 5 narration sections
- `work/audio/*.wav` - Voice-cloned audio (~32 sec)
- `work/slides/*.png` - Title slides
- `work/clips/*.mp4` - Video segments
- `output/preview.mp4` - Final video

## Step Kinds

| Kind | Description |
|------|-------------|
| `ensure_dirs` | Create directories |
| `write_file` | Write templated content |
| `split_sections` | Split LLM output by headings |
| `run_command` | Execute shell command |
| `llm_generate` | Generate text via LLM |
| `tts_generate` | Voice clone via VoxCPM |

## Recent Changes

- **GPU Queue:** Semaphore-based task serialization for TTS/lipsync
- **TTS Step:** tts_generate step kind with VoxCPM integration
- **Resume Support:** --resume flag skips completed steps
- **Output Validation:** Media duration checking via ffprobe
- **Demo Scripts:** generate-tts.sh, verify-tts.sh, build-video.sh
- **HOWTO.md:** Full pipeline documentation

## Dependency Graph

```
vwf-types (L0)
    |
    v
vwf-runtime + vwf-dag + vwf-queue (L1)
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
3. Lipsync step not yet implemented (queue ready)

## Next Steps

1. **Real LLM Integration** - Claude API adapter
2. **Lipsync Step** - MuseTalk integration using lipsync queue
3. **Avatar Compositing** - vid-composite integration
4. **Full Video Pipeline** - End-to-end with all steps
