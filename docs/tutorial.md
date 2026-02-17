# VWF Tutorial

This tutorial explains how to create and run video workflows with VWF.

## What is a Workflow?

A workflow is a YAML file that defines a sequence of automated steps. Each step performs a specific action: creating directories, writing files, generating text with an LLM, creating TTS audio, or running shell commands.

## Workflow Structure

```yaml
version: 1
name: "My Workflow"
description: >
  A brief description of what this workflow does.

vars:
  project_name: "My Project"
  style: "professional"

steps:
  - id: step_1
    kind: ensure_dirs
    dirs: ["work", "output"]

  - id: step_2
    kind: write_file
    path: "work/file.txt"
    content: "Hello {{project_name}}"
```

### Top-Level Fields

| Field | Required | Description |
|-------|----------|-------------|
| `version` | Yes | Always `1` |
| `name` | Yes | Human-readable workflow name |
| `description` | No | What the workflow does |
| `vars` | No | Variables for template substitution |
| `steps` | Yes | List of steps to execute |

### Variables and Templates

Variables defined in `vars` can be used in step fields with `{{variable_name}}` syntax:

```yaml
vars:
  topic: "Machine Learning"
  audience: "beginners"

steps:
  - id: write_prompt
    kind: write_file
    path: "work/prompt.txt"
    content: |
      Write about {{topic}} for {{audience}}.
```

## Step Kinds

### ensure_dirs

Creates directories if they don't exist.

```yaml
- id: setup
  kind: ensure_dirs
  dirs:
    - "work/scripts"
    - "work/audio"
    - "output"
```

### write_file

Writes content to a file. Supports template variables.

```yaml
- id: write_prompt
  kind: write_file
  path: "work/prompt.txt"
  content: |
    Topic: {{topic}}
    Style: {{style}}
```

### llm_generate

Generates text using an LLM. Currently supports mock provider.

```yaml
- id: generate_script
  kind: llm_generate
  system: "You write clear, concise narration."
  user_prompt_path: "work/prompt.txt"
  output_path: "work/script.txt"
  provider: "mock"
  mock_response: |
    This is the mock response that will be written.
```

| Field | Required | Description |
|-------|----------|-------------|
| `system` | Yes | System prompt for the LLM |
| `user_prompt_path` | Yes | Path to file containing user prompt |
| `output_path` | Yes | Where to write LLM response |
| `provider` | Yes | LLM provider (`mock` for now) |
| `mock_response` | No | Canned response for mock provider |

### split_sections

Splits a text file by headings into multiple files.

```yaml
- id: split_sections
  kind: split_sections
  input_path: "work/script.txt"
  outputs:
    - heading: "HOOK:"
      path: "work/scripts/01-hook.txt"
    - heading: "CONTENT:"
      path: "work/scripts/02-content.txt"
    - heading: "CTA:"
      path: "work/scripts/03-cta.txt"
```

### run_command

Executes a shell command.

```yaml
- id: build_video
  kind: run_command
  command: "./scripts/build-video.sh"
  workdir: "."
```

### tts_generate

Generates TTS audio using VoxCPM voice cloning.

```yaml
- id: tts_hook
  kind: tts_generate
  script_path: "work/scripts/01-hook.txt"
  output_path: "work/audio/01-hook.wav"
  server: "http://curiosity:7860"
  reference_audio: "/path/to/reference.wav"
  reference_text: "Transcript of reference audio..."
```

| Field | Required | Description |
|-------|----------|-------------|
| `script_path` | Yes | Path to text file with narration |
| `output_path` | Yes | Where to write audio file |
| `server` | No | VoxCPM server URL (default: http://curiosity:7860) |
| `reference_audio` | Yes | Path to voice reference WAV |
| `reference_text` | Yes | Transcript of reference audio |

### text_to_image

Generates images using FLUX.1 schnell via ComfyUI.

```yaml
- id: gen_background
  kind: text_to_image
  prompt: "A futuristic coding workspace, dark theme, neon accents"
  output_path: "work/images/background.png"
  orientation: "landscape"  # portrait, landscape, or square
  server: "http://192.168.1.64:8570"
```

| Field | Required | Description |
|-------|----------|-------------|
| `prompt` | Yes | Text prompt for image generation |
| `output_path` | Yes | Where to save the image |
| `orientation` | No | portrait (720x1280), landscape (1280x720), square (1024x1024) |
| `width` | No | Custom width (overrides orientation) |
| `height` | No | Custom height (overrides orientation) |
| `seed` | No | Random seed for reproducibility |
| `server` | No | ComfyUI server URL (default: http://192.168.1.64:8570) |

## Standalone GPU Tools

Beyond workflow steps, these Python scripts provide direct access to GPU services.

### Image-to-Video (SVD)

Animate a still image using Stable Video Diffusion. Best for natural motion.

```bash
python scripts/svd_client.py \
  --input image.jpg \
  --output video.mp4 \
  --frames 30 \
  --fps 6 \
  --motion 100
```

| Field | Default | Description |
|-------|---------|-------------|
| `--input` | Required | Source image |
| `--output` | Required | Output video path |
| `--frames` | 14 | Frame count (30 = 5s @ 6fps) |
| `--fps` | 6 | Frame rate |
| `--motion` | 127 | Motion intensity 1-300 (higher = more motion) |
| `--server` | http://192.168.1.64:8100 | SVD ComfyUI server |

**Best subjects for SVD:** Water, fire, clouds, foliage, portraits with hair.
**Avoid:** Geometric scenes, forward camera travel, complex physics.

### Text-to-Video (Wan 2.2)

Generate video directly from text prompts. No input image needed.

```bash
python scripts/wan22_client.py \
  --prompt "A serene forest at sunrise, mist rising" \
  --output video.mp4 \
  --orientation landscape \
  --length 81
```

| Field | Default | Description |
|-------|---------|-------------|
| `--prompt` | Required | Text description |
| `--output` | Required | Output video path |
| `--orientation` | landscape | portrait, landscape, or square |
| `--length` | 81 | Frame count (81 = ~5s @ 16fps) |
| `--cfg` | 5.0 | Prompt adherence (4-7 recommended) |
| `--server` | http://192.168.1.64:6000 | Wan 2.2 ComfyUI server |

**Generation time:** ~13 minutes for 81 frames (5 seconds).
**Output resolution:** 2x latent size (landscape: 1664x960, portrait: 960x1664).

### Music Generation (midi-cli-rs)

Generate incidental music for intros, outros, and backgrounds.

```bash
midi-cli-rs preset --mood upbeat --duration 5 -o intro.wav
midi-cli-rs preset --mood calm --duration 5 -o outro.wav
midi-cli-rs preset --mood suspense --duration 5 -o dramatic.wav
midi-cli-rs preset --mood ambient --duration 5 -o background.wav
midi-cli-rs preset --mood eerie --duration 5 -o creepy.wav
```

| Mood | Key | Description |
|------|-----|-------------|
| upbeat | C | Energetic, rhythmic patterns |
| calm | G | Peaceful, sustained pads |
| suspense | Am | Tense, low drones |
| ambient | Em | Atmospheric, pentatonic |
| eerie | Dm | Creepy, sparse tones |

Use `--seed` for reproducible output: `midi-cli-rs preset -m upbeat -d 5 --seed 42 -o intro.wav`

## Running Workflows

### Basic Run

```bash
cargo run -p vwf-cli -- run workflow.yaml --workdir ./my-project
```

### Resume Mode

Skip steps whose outputs already exist:

```bash
cargo run -p vwf-cli -- run workflow.yaml --workdir ./my-project --resume
```

For resume to work, add `resume_output` to steps that produce files:

```yaml
- id: generate_audio
  kind: tts_generate
  resume_output: "work/audio/narration.wav"  # Skip if this exists
  script_path: "work/scripts/narration.txt"
  output_path: "work/audio/narration.wav"
  # ...
```

### Dry Run

See what would happen without executing:

```bash
cargo run -p vwf-cli -- run workflow.yaml --workdir ./my-project --dry-run
```

## Video Formats

VWF supports two video formats:

| Format | Dimensions | Orientation | Duration |
|--------|-----------|-------------|----------|
| **YouTube Short** | 1080x1920 | Vertical (9:16) | < 60 sec (max 3 min) |
| **Explainer** | 1920x1080 | Landscape (16:9) | 1.5-30 min (ideal 5-10) |

## Example Workflows

### YouTube Short

Location: `examples/workflows/shorts_narration.yaml`

A 3-section structure optimized for vertical format:
- **Hook** (~10 sec): Attention-grabbing question
- **Content** (~10 sec): Key insight or value
- **CTA** (~10 sec): Call to action

```bash
# Run the demo
./scripts/demo-short.sh
```

See `test-projects/sample-short/` for a complete working example.

### Explainer Video

Location: `examples/workflows/explainer_outline.yaml`

A 5-section structure for longer content:
- **Hook**: Grab attention
- **Problem**: Pain point
- **Solution**: What solves it
- **Benefit**: Why it matters
- **CTA**: Call to action

```bash
# Run the demo
./scripts/demo-explainer.sh
```

See `test-projects/sample-video/` for a complete working example.

## Project Structure

A typical VWF project looks like:

```
my-project/
├── workflow.yaml           # Workflow definition
├── scripts/
│   ├── common.sh          # Shared variables
│   ├── generate-tts.sh    # TTS generation
│   ├── verify-tts.sh      # Whisper verification
│   └── build-video.sh     # Video assembly
├── work/                   # Working directory (generated)
│   ├── scripts/           # Narration text files
│   ├── audio/             # TTS audio files
│   ├── slides/            # Slide images
│   └── clips/             # Video segments
└── output/                 # Final output
    └── preview.mp4
```

## Workflow Development Tips

1. **Start with mock LLM**: Use `provider: "mock"` with `mock_response` to develop without API calls.

2. **Use resume**: Add `resume_output` to expensive steps (TTS, LLM) so you can iterate without regenerating everything.

3. **Verify TTS**: Always run whisper verification after TTS to catch pronunciation issues.

4. **Phonetic fixes**: For TTS mispronunciations, use phonetic spelling:
   - "run it" → "runn it"
   - "error-prone" → "prone to errors"
   - "missed step" → "missed, step" (adds pause)

5. **Test incrementally**: Run with `--dry-run` first, then without `--resume` for fresh output, then with `--resume` for iterating.

## Quick Reference

```bash
# Generate a YouTube Short
./scripts/demo-short.sh

# Generate an explainer
./scripts/demo-explainer.sh

# Run any workflow
cargo run -p vwf-cli -- run workflow.yaml --workdir ./project

# Resume interrupted workflow
cargo run -p vwf-cli -- run workflow.yaml --workdir ./project --resume

# Dry run
cargo run -p vwf-cli -- run workflow.yaml --workdir ./project --dry-run
```

## GPU Services

Three ComfyUI services share a single GPU. Run one at a time to avoid OOM.

| Service | Port | Model | Use Case |
|---------|------|-------|----------|
| FLUX | 8570 | flux1-schnell-fp8 | Text → Image |
| SVD | 8100 | svd_xt | Image → Video |
| Wan 2.2 | 6000 | wan2.2_ti2v_5B | Text → Video |

### Quick Test

```bash
# Check which service is running
curl -s http://192.168.1.64:8570/system_stats  # FLUX
curl -s http://192.168.1.64:8100/system_stats  # SVD
curl -s http://192.168.1.64:6000/system_stats  # Wan 2.2
```

### Service Management (from server host)

```bash
# Stop SVD, start FLUX
cd ~/tools/tti/ittv && docker compose down
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy up -d

# Stop FLUX, start SVD
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy down
cd ~/tools/tti/ittv && docker compose up -d

# Stop SVD, start Wan 2.2
cd ~/tools/tti/ittv && docker compose down
cd ~/tools/tti/comfy-wan22 && docker compose up -d
```

See `docs/flux-client-usage.md`, `docs/svd-client-usage.md`, and `docs/wan22-client-usage.md` for detailed API documentation.
