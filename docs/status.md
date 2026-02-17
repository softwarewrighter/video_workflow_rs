# Project Status

## Current State: Full Media Generation Pipeline

All components pass sw-checklist (0 failures). Full video pipeline working: script generation, TTS voice cloning, video assembly. GPU services available for text-to-image, image-to-video, and text-to-video generation.

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
# Generate a YouTube Short (vertical 1080x1920, ~30 sec)
./scripts/demo-short.sh

# Generate an explainer video (landscape 1920x1080, ~32 sec)
./scripts/demo-explainer.sh
```

## Video Formats

| Format | Dimensions | Orientation | Duration |
|--------|-----------|-------------|----------|
| **YouTube Short** | 1080x1920 | Vertical (9:16) | < 60 sec (max 3 min) |
| **Explainer** | 1920x1080 | Landscape (16:9) | 1.5-30 min (ideal 5-10) |

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

Steps declare `resume_output` for completion checking. Media files validated via ffprobe duration.

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

## Test Projects

### sample-short (YouTube Short)

**Location:** `test-projects/sample-short/`
**Format:** 1080x1920 vertical (9:16)
**Duration:** ~30 seconds

3-section structure: Hook, Content, CTA. Uses ImageMagick for vertical slides.

### sample-video (Explainer)

**Location:** `test-projects/sample-video/`
**Format:** 1920x1080 landscape (16:9)
**Duration:** ~32 seconds

5-section structure: Hook, Problem, Solution, Benefit, CTA. Uses vid-slide for slides.

**Common Scripts (in each project):**
- `scripts/generate-tts.sh` - Generate voice-cloned audio
- `scripts/verify-tts.sh` - Verify with Whisper
- `scripts/build-video.sh` - Assemble final video

## Step Kinds

| Kind | Description |
|------|-------------|
| `ensure_dirs` | Create directories |
| `write_file` | Write templated content |
| `split_sections` | Split LLM output by headings |
| `run_command` | Execute shell command |
| `llm_generate` | Generate text via LLM |
| `tts_generate` | Voice clone via VoxCPM |
| `text_to_image` | Generate image via FLUX |

## GPU Services (Remote ComfyUI)

Three GPU services share a single NVIDIA RTX 5060 Ti 16GB. Run one at a time.

| Service | Port | Model | Use Case | Gen Time |
|---------|------|-------|----------|----------|
| **FLUX** | 8570 | flux1-schnell-fp8 | Text → Image | ~12s |
| **SVD** | 8100 | svd_xt | Image → Video | ~70s (14 frames) |
| **Wan 2.2** | 6000 | wan2.2_ti2v_5B | Text → Video | ~13min (81 frames) |

### Client Scripts

```bash
# Text-to-image (FLUX)
python scripts/flux_client.py -p "prompt" -o image.png --orientation portrait

# Image-to-video (SVD) - best for natural motion (water, fire, foliage)
python scripts/svd_client.py -i image.jpg -o video.mp4 --motion 100 --frames 30

# Text-to-video (Wan 2.2) - true text-to-video generation
python scripts/wan22_client.py -p "prompt" -o video.mp4 --orientation landscape

# Demo: Generate images in all orientations
./scripts/demo-flux.sh
```

### SVD Best Practices

SVD works best with natural/organic motion. Avoid complex physics or geometry.

| Works Well | Avoid |
|------------|-------|
| Water, waves, ripples | Forward camera travel |
| Fire, smoke, candles | Architectural scenes |
| Foliage, grass, wind | Complex object physics |
| Clouds, sky, aurora | Rotating camera |
| Subtle zoom/parallax | Action sequences |

## Music Generation (midi-cli-rs)

Generate incidental music for intros/outros:

```bash
midi-cli-rs preset --mood upbeat --duration 5 -o intro.wav
midi-cli-rs preset --mood calm --duration 5 -o outro.wav
midi-cli-rs preset --mood suspense --duration 5 -o dramatic.wav
midi-cli-rs preset --mood ambient --duration 5 -o background.wav
midi-cli-rs preset --mood eerie --duration 5 -o creepy.wav

# Reproducible with seed
midi-cli-rs preset -m upbeat -d 5 --seed 42 -o intro.wav
```

## Recent Changes

- **Text-to-Image:** FLUX integration via ComfyUI (flux_client.py, text_to_image step)
- **Image-to-Video:** SVD integration via ComfyUI (svd_client.py)
- **Text-to-Video:** Wan 2.2 integration via ComfyUI (wan22_client.py)
- **Music Generation:** midi-cli-rs integration for incidental music
- **GPU Queue:** Semaphore-based task serialization for TTS/lipsync
- **TTS Step:** tts_generate step kind with VoxCPM integration
- **Resume Support:** --resume flag skips completed steps
- **Output Validation:** Media duration checking via ffprobe
- **Demo Scripts:** demo-flux.sh, generate-tts.sh, verify-tts.sh, build-video.sh

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
4. GPU services share one GPU — run one at a time to avoid OOM
5. SVD struggles with geometric scenes and complex camera motion

## Next Steps

1. **Real LLM Integration** - Claude API adapter
2. **Lipsync Step** - MuseTalk integration using lipsync queue
3. **Avatar Compositing** - vid-composite integration
4. **Image-to-Video Step** - Native svd_generate workflow step
5. **Text-to-Video Step** - Native wan22_generate workflow step
6. **Music Step** - Native midi_generate workflow step
