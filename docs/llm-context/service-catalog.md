# VWF Service Catalog

This document describes the AI services available for video production workflows.

## Service Overview

| Service | Purpose | Step Types | Default Endpoint |
|---------|---------|------------|------------------|
| Ollama | Local LLM text generation | llm_generate | http://localhost:11434 |
| VoxCPM | Voice cloning TTS | tts_generate | http://curiosity:7860 |
| FLUX.1 | Text-to-image generation | text_to_image | http://192.168.1.64:8570 |
| SVD-XT | Image-to-video animation | image_to_video | http://192.168.1.64:8100 |
| Wan 2.2 | Text-to-video generation | text_to_video | http://192.168.1.64:6000 |
| Whisper | Speech-to-text | whisper_transcribe | (local CLI) |
| midi-cli-rs | Music generation | run_command | (local CLI) |

---

## Ollama (Local LLM)

**Purpose:** Generate text content using local large language models.

**Typical Uses:**
- Generate image prompts from descriptions
- Write narration scripts
- Create video titles and descriptions
- Summarize content

**Available Models:**
- `qwen2.5-coder:14b` - Best for technical content (recommended)
- `gemma2:9b` - Good general purpose
- `llama3.2:latest` - Fast, versatile

**Usage:**
```bash
# Run workflow with Ollama
vwf run workflow.yaml --workdir project --llm-model qwen2.5-coder:14b
```

**Example Step:**
```yaml
- id: write_prompt
  kind: llm_generate
  system_prompt: "You write concise FLUX.1 image prompts."
  user_prompt: "A professional software developer at work"
  output_path: "work/prompts/dev.txt"
```

---

## VoxCPM (Text-to-Speech)

**Purpose:** Generate natural-sounding speech with voice cloning.

**Capabilities:**
- Clone any voice from 20-60 second reference
- High quality speech synthesis
- Multiple languages supported

**Server:** http://curiosity:7860 (Gradio API)

**Requirements:**
- Reference audio WAV (20-60 seconds, clear speech)
- Transcript of reference audio

**Example Step:**
```yaml
- id: narrate_intro
  kind: tts_generate
  script_path: "work/scripts/intro.txt"
  output_path: "work/audio/intro.wav"
  reference_audio: "assets/voice-reference.wav"
  reference_text: "This is my voice sample for cloning."
  server: "http://curiosity:7860"
```

**Best Practices:**
- Use clean reference audio without background noise
- Match reference speaker's style to desired output
- Keep narration scripts under 500 words per segment

---

## FLUX.1 (Text-to-Image)

**Purpose:** Generate high-quality images from text prompts.

**Model:** FLUX.1 schnell (fast, 4-step generation)

**Server:** http://192.168.1.64:8570 (ComfyUI API)

**Supported Resolutions:**
- Portrait: 720x1280 (9:16)
- Landscape: 1280x720 (16:9)
- Square: 1024x1024

**Example Step:**
```yaml
- id: hero_image
  kind: text_to_image
  prompt: "Professional developer at modern desk, warm lighting, photorealistic"
  output_path: "work/images/hero.png"
  orientation: landscape
  seed: 2026
```

**Prompt Tips:**
- Be specific about style, lighting, and composition
- Include quality modifiers: "photorealistic", "professional", "high quality"
- Avoid negative terms (use positive descriptions instead)

---

## SVD-XT (Image-to-Video)

**Purpose:** Animate still images into short video clips.

**Model:** Stable Video Diffusion XT

**Server:** http://192.168.1.64:8100 (ComfyUI API)

**Capabilities:**
- 14 frames at 6fps (~2.3 seconds)
- Motion intensity control (1-1023)
- Preserves image composition

**Example Step:**
```yaml
- id: animate_hero
  kind: image_to_video
  input_path: "work/images/hero.png"
  output_path: "work/videos/hero.mp4"
  frames: 14
  fps: 6
  motion: 80  # Subtle motion
```

**Motion Guidelines:**
- 40-80: Subtle breathing/ambient motion
- 100-200: Moderate movement
- 300+: Dramatic motion (may distort)

---

## Wan 2.2 (Text-to-Video)

**Purpose:** Generate video directly from text prompts.

**Model:** Wan 2.2 Ti2V 5B

**Server:** http://192.168.1.64:6000 (ComfyUI API)

**Capabilities:**
- Up to 81 frames at 16fps (~5 seconds)
- Text-to-video from scratch
- High quality motion

**Generation Time:** ~13 minutes for 81 frames on RTX 4090

**Example Step:**
```yaml
- id: title_video
  kind: text_to_video
  prompt: "A friendly cartoon dog eating from a bowl, tech office background"
  output_path: "work/videos/title.mp4"
  orientation: landscape
  length: 81
  steps: 20
  cfg: 5.0
```

**Prompt Tips:**
- Describe motion explicitly: "walking", "typing", "looking around"
- Include setting and lighting details
- Keep prompts focused on single actions

---

## Whisper (Speech-to-Text)

**Purpose:** Transcribe audio to text.

**Implementation:** Local whisper CLI

**Available Models:**
- tiny, base, small, medium, large
- Default: base (good accuracy/speed balance)

**Example Step:**
```yaml
- id: transcribe
  kind: whisper_transcribe
  input_path: "work/audio/narration.wav"
  output_path: "work/transcripts/narration.txt"
  model: "base"
```

---

## midi-cli-rs (Music Generation)

**Purpose:** Generate background music from MIDI parameters.

**Implementation:** Local Rust binary with soundfonts

**Parameters:**
- mood: upbeat, calm, suspense, ambient, eerie
- seed: Reproducibility control
- duration: Length in seconds

**Example Step:**
```yaml
- id: generate_music
  kind: run_command
  program: midi-cli-rs
  args:
    - "--mood"
    - "ambient"
    - "--seed"
    - "2026"
    - "--duration"
    - "120"
    - "--output"
    - "work/audio/background.wav"
```

**Note:** Requires `--allow midi-cli-rs` flag when running workflow.

---

## Service Health Checking

Before running a workflow, verify services are available:

```bash
# Check VoxCPM
curl -s http://curiosity:7860/api/predict -o /dev/null && echo "VoxCPM: OK"

# Check FLUX.1
curl -s http://192.168.1.64:8570/system_stats && echo "FLUX.1: OK"

# Check SVD-XT
curl -s http://192.168.1.64:8100/system_stats && echo "SVD-XT: OK"

# Check Wan 2.2
curl -s http://192.168.1.64:6000/system_stats && echo "Wan 2.2: OK"

# Check Ollama
curl -s http://localhost:11434/api/tags && echo "Ollama: OK"
```

---

## Network Topology

```
Local Machine (vwf-cli)
├── Ollama (localhost:11434) - LLM generation
├── midi-cli-rs (local binary) - Music
└── whisper (local binary) - Transcription

GPU Server (192.168.1.64)
├── ComfyUI :8570 - FLUX.1 (text-to-image)
├── ComfyUI :8100 - SVD-XT (image-to-video)
└── ComfyUI :6000 - Wan 2.2 (text-to-video)

TTS Server (curiosity)
└── VoxCPM :7860 - Voice cloning TTS
```
