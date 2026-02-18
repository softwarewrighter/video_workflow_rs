# Project Status

## Current State: Full Production Pipeline with DAG Execution

All components pass tests. Full video pipeline working with DAG-based execution: script generation, LLM integration (Ollama), TTS voice cloning, image/video generation, and web status viewer. Steps run based on dependency satisfaction, with clear blocked step reporting.

## Last Updated: 2026-02-18

## Milestone Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Workflow Runner | Complete | 100% |
| M2: Shell Step | Complete | 100% |
| M3: LLM Adapter (Ollama) | Complete | 100% |
| M4: Web UI | Functional | 60% (status viewer + editor) |
| M5: Component Restructure | Complete | 100% |
| M6: sw-checklist Compliance | Complete | 100% |
| M7: Sample Video Pipeline | Complete | 100% |
| M8: GPU Queue + Resume | Complete | 100% |
| M9: DAG Execution | Complete | 100% |
| M10: Service Health Check | Complete | 100% |

## Quick Start

```bash
# Build all components
./scripts/build.sh

# Run tests
./scripts/test.sh

# Run a workflow with resume (skips completed steps)
./scripts/run.sh projects/self/workflow.yaml --resume --allow bash --allow ffmpeg

# Check service availability
cd components/vwf-apps && cargo run -p vwf-cli -- services ../../projects/self/workflow.yaml

# Serve web UI
./scripts/serve-web.sh
```

## New Features

### DAG-Based Execution

Steps declare dependencies with `depends_on` and run when dependencies are satisfied:

```yaml
steps:
  - id: setup_dirs
    kind: ensure_dirs
    dirs: ["work/audio", "work/videos"]

  - id: generate_script
    kind: write_file
    depends_on: [setup_dirs]
    path: "work/scripts/intro.txt"
    content: "Welcome to the demo."

  - id: tts_narration
    kind: tts_generate
    depends_on: [generate_script]
    # ...
```

Features:
- **Cycle detection** before execution
- **Blocked step tracking** when dependencies fail
- **Clear status reporting**: OK, Skipped, Failed, Blocked

### Web Status Viewer

Load `run.json` files in the web UI to view execution status:

```bash
./scripts/serve-web.sh
# Open http://localhost:8090, click Status tab, load run.json
```

Shows:
- Summary badges (OK/Skipped/Failed/Blocked counts)
- Step table with color-coded status
- Error messages for failed steps

### LLM Integration (Ollama)

Real LLM generation via local Ollama:

```yaml
- id: generate_prompt
  kind: llm_generate
  system_prompt: "You are a helpful assistant."
  user_prompt: "Write a greeting."
  output_path: "work/prompts/greeting.txt"
```

Run with: `--llm-model qwen2.5-coder:14b`

### Service Health Check

Check availability of required services:

```bash
vwf services projects/self/workflow.yaml
```

Output shows which services are running/offline and how to start them.

## Component Structure

```
components/
|-- vwf-foundation/   # 4 crates
|   |-- vwf-types
|   |-- vwf-runtime   # FsRuntime, OllamaClient
|   |-- vwf-dag
|   +-- vwf-queue     # GPU semaphores
|-- vwf-engine/       # 4 crates
|   |-- vwf-config    # StepKind, depends_on
|   |-- vwf-render    # Template {{vars}}
|   |-- vwf-steps     # 15 step implementations
|   +-- vwf-core      # DAG engine, reports
+-- vwf-apps/         # 2 crates
    |-- vwf-cli       # run, show, generate, services
    +-- vwf-web       # Yew WASM UI
```

## Step Kinds

| Kind | Description |
|------|-------------|
| `ensure_dirs` | Create directories |
| `write_file` | Write templated content |
| `split_sections` | Split text by delimiter |
| `run_command` | Execute shell command (requires --allow) |
| `llm_generate` | Generate text via Ollama |
| `llm_audit` | Audit assets with vision LLM |
| `tts_generate` | Voice clone via VoxCPM |
| `whisper_transcribe` | Transcribe audio via Whisper |
| `text_to_image` | Generate image via FLUX |
| `image_to_video` | Animate image via SVD-XT |
| `text_to_video` | Generate video via Wan 2.2 |
| `normalize_volume` | Normalize audio to target dB |
| `audio_mix` | Mix overlay audio onto video |
| `video_concat` | Concatenate video clips |
| `create_slide` | Generate title/text slides |

## GPU Services (Remote ComfyUI)

| Service | Port | Model | Use Case |
|---------|------|-------|----------|
| **FLUX** | 8570 | flux1-schnell-fp8 | Text to Image |
| **SVD** | 8100 | svd_xt | Image to Video |
| **Wan 2.2** | 6000 | wan2.2_ti2v_5B | Text to Video |

## Development Scripts

| Script | Description |
|--------|-------------|
| `./scripts/build.sh` | Build all components |
| `./scripts/test.sh` | Run all tests + Python hygiene |
| `./scripts/lint.sh` | Run clippy (zero warnings) |
| `./scripts/fmt.sh` | Format code |
| `./scripts/run.sh` | Run a workflow |
| `./scripts/show.sh` | Debug workflow parsing |
| `./scripts/serve-web.sh` | Serve web UI |
| `./scripts/setup-python.sh` | Set up Python venv |

## Known Issues

1. GPU services share one GPU - run one at a time to avoid OOM
2. SVD struggles with geometric scenes and complex camera motion
3. TTS requires VoxCPM server running on curiosity
4. Some ComfyUI workflows may need model downloads on first run

## Next Steps

1. **Self-Explainer Video** - Complete dogfooding demo
2. **Real-time Log Streaming** - WebSocket logs in web UI
3. **Service Health Panel** - Live status in web UI
4. **LLM Context Documents** - Better `vwf generate` prompts
5. **Lipsync Step** - MuseTalk integration
