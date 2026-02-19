# video-workflow-rs (VWF)

A Rust framework for *repeatable* video-production workflows with strict, minimal-context AI calls.

## Goals

- **Inversion of control:** workflows are data (YAML), the runner is code.
- **Determinism & auditability:** every run produces a manifest (`run.json`) + artifacts.
- **AI calls as "steps":** prompts are templated; responses are captured and validated.
- **DAG execution:** steps run in parallel when dependencies allow.
- **TDD-friendly:** core logic is unit-tested with mocks (no network, no shell by default).
- **Extensible:** add new step kinds without rewriting the runner.

## Quick Start

```bash
# Build and test
cd components/vwf-engine && cargo test
cd components/vwf-apps && cargo test

# Dry run (preview without executing)
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo --dry-run

# Full run with incremental builds
cargo run -p vwf-cli -- run workflow.yaml --workdir project --resume --allow ffmpeg --allow midi-cli-rs

# Check service availability
cargo run -p vwf-cli -- services workflow.yaml
```

## Project Structure

```
components/
├── vwf-engine/          # Core library crates
│   └── crates/
│       ├── vwf-config/  # YAML parsing, StepKind enum
│       ├── vwf-core/    # Workflow engine, DAG execution
│       ├── vwf-render/  # Template variable substitution
│       └── vwf-steps/   # Step implementations (15 types)
└── vwf-apps/            # Application crates
    └── crates/
        ├── vwf-cli/     # Command-line interface
        └── vwf-web/     # Yew/WASM status viewer

projects/                # Video project workspaces
docs/                    # Documentation
examples/                # Example workflows
```

## Step Types

| Category | Steps |
|----------|-------|
| Core | `ensure_dirs`, `write_file`, `run_command`, `split_sections` |
| AI Generation | `llm_generate`, `tts_generate`, `text_to_image`, `image_to_video`, `text_to_video` |
| Audio | `normalize_volume`, `audio_mix`, `whisper_transcribe` |
| Video | `video_concat`, `create_slide` |
| Quality | `llm_audit` |

## Dependencies

### Required

- **Rust** (stable, 2024 edition)
- **FFmpeg** - video/audio processing

### Optional Services

| Service | Purpose | Default Endpoint |
|---------|---------|------------------|
| Ollama | Local LLM | http://localhost:11434 |
| VoxCPM | Voice cloning TTS | http://curiosity:7860 |
| FLUX.1 | Text-to-image | http://192.168.1.64:8570 |
| SVD-XT | Image-to-video | http://192.168.1.64:8100 |
| Wan 2.2 | Text-to-video | http://192.168.1.64:6000 |

### Music Generation (midi-cli-rs)

For WAV audio output from `midi-cli-rs`, you need FluidSynth and a SoundFont:

```bash
# macOS
brew install fluid-synth

# Ubuntu/Debian
sudo apt install fluidsynth fluid-soundfont-gm

# Find your SoundFont location
find /opt/homebrew /usr/share -name "*.sf2" 2>/dev/null | head -5
```

Use the `--soundfont` flag in your workflow. Define a variable for the path:

```yaml
# In workflow.yaml
vars:
  soundfont: "/opt/homebrew/Cellar/fluid-synth/2.5.2/share/fluid-synth/sf2/VintageDreamsWaves-v2.sf2"

steps:
  - id: music
    kind: run_command
    program: midi-cli-rs
    args: ["preset", "--mood", "upbeat", "--soundfont", "{{soundfont}}", "-o", "output.wav"]
```

Common SoundFont locations:
- **macOS (Homebrew):** `/opt/homebrew/Cellar/fluid-synth/*/share/fluid-synth/sf2/*.sf2`
- **Ubuntu:** `/usr/share/sounds/sf2/FluidR3_GM.sf2`

Without a SoundFont, `midi-cli-rs` outputs MIDI files only (no WAV rendering).

## DAG Execution

Steps declare dependencies with `depends_on`. Independent steps run in parallel:

```yaml
steps:
  - id: setup
    kind: ensure_dirs
    dirs: ["work/audio", "work/images"]

  # These run in parallel (both only depend on setup)
  - id: generate_audio
    kind: tts_generate
    depends_on: [setup]
    # ...

  - id: generate_image
    kind: text_to_image
    depends_on: [setup]
    # ...

  # This waits for both
  - id: combine
    kind: run_command
    depends_on: [generate_audio, generate_image]
    # ...
```

Failed steps mark dependents as "blocked" without stopping unrelated work.

## Incremental Builds

Use `resume_output` to skip expensive steps when output exists:

```yaml
- id: expensive_step
  kind: text_to_video
  resume_output: "work/videos/title.mp4"  # Skip if exists
  output_path: "work/videos/title.mp4"
```

Run with `--resume` flag to enable.

## Documentation

- [CLAUDE.md](CLAUDE.md) - AI coding agent instructions
- [docs/llm-context/](docs/llm-context/) - LLM reference documents for workflow generation
- [docs/status.md](docs/status.md) - Project status and roadmap

## Philosophy

Your agents are unreliable because *the world is huge* and "helpful guessing" is the default failure mode. This project forces:

- Small prompts
- Pinned parameters
- Explicit inputs/outputs per step
- Validation gates
- Provenance logs (`run.json`)

## License

Copyright 2026 Software Wrighter LLC. MIT License.
