# VWF Services Reference

This document describes the external services VWF integrates with for asset generation.

## Service Overview

| Service | Type | Default Port | Model |
|---------|------|--------------|-------|
| VoxCPM | TTS (voice cloning) | 7860 | Custom voice clone |
| FLUX.1 schnell | Text-to-Image | 8570 | flux1-schnell-fp8.safetensors |
| SVD-XT | Image-to-Video | 8100 | svd_xt.safetensors |
| Wan 2.2 | Text-to-Video | 6000 | wan2.2_ti2v_5B_fp16.safetensors |
| midi-cli-rs | Music Generation | N/A (local) | VintageDreamsWaves-v2.sf2 |
| whisper.cpp | Transcription | N/A (local) | ggml-base.en.bin |
| Ollama | LLM Text Generation | 11434 | qwen2.5-coder:14b |

## VoxCPM (Text-to-Speech)

Voice cloning TTS service that generates natural speech from text using a reference audio sample.

**Endpoint**: `http://curiosity:7860`

**Step Type**: `tts_generate`

```yaml
- id: tts_intro
  kind: tts_generate
  resume_output: "work/audio/intro.wav"
  script_path: "work/scripts/intro.txt"
  output_path: "work/audio/intro.wav"
  server: "{{tts_server}}"
  reference_audio: "{{reference_audio}}"
  reference_text: "{{reference_text}}"
  python_path: "{{python_path}}"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `script_path` | Yes | Path to text file containing narration script |
| `output_path` | Yes | Output WAV file path |
| `server` | Yes | VoxCPM server URL |
| `reference_audio` | Yes | Path to reference voice WAV file (~30s recommended) |
| `reference_text` | Yes | Transcript of the reference audio |
| `python_path` | No | Python interpreter path (default: python3) |

**Reference Audio Guidelines**:
- Duration: 20-60 seconds of clear speech
- Quality: Clean recording, minimal background noise
- Content: Natural conversational speech with varied intonation
- Format: WAV, 16-bit, mono or stereo

## FLUX.1 schnell (Text-to-Image)

Fast text-to-image generation using the FLUX.1 schnell model via ComfyUI.

**Endpoint**: `http://192.168.1.64:8570`

**Model**: `flux1-schnell-fp8.safetensors`

**Step Type**: `text_to_image`

```yaml
- id: img_background
  kind: text_to_image
  resume_output: "work/images/background.png"
  prompt: "Abstract tech visualization, glowing blue nodes, dark background"
  output_path: "work/images/background.png"
  orientation: "landscape"
  seed: 42001
  server: "http://192.168.1.64:8570"
  python_path: "{{python_path}}"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `prompt` | Yes | Text description of desired image |
| `output_path` | Yes | Output PNG file path |
| `orientation` | No | `portrait` (720x1280), `landscape` (1280x720), `square` (1024x1024) |
| `seed` | No | Random seed for reproducibility |
| `server` | No | ComfyUI server URL (default: http://192.168.1.64:8570) |
| `python_path` | No | Python interpreter path |

**Output Resolutions**:

| Orientation | Resolution | Aspect Ratio |
|-------------|------------|--------------|
| landscape | 1280x720 | 16:9 |
| portrait | 720x1280 | 9:16 |
| square | 1024x1024 | 1:1 |

**Prompt Tips**:
- Be specific about style: "photorealistic", "digital art", "minimalist"
- Include lighting: "soft backlighting", "neon glow", "dark moody"
- Avoid text in images (AI struggles with text rendering)

## SVD-XT (Image-to-Video)

Stable Video Diffusion XT generates short video clips from static images via ComfyUI.

**Endpoint**: `http://192.168.1.64:8100`

**Model**: `svd_xt.safetensors`

**Step Type**: `image_to_video`

```yaml
- id: vid_animated
  kind: image_to_video
  resume_output: "work/videos/animated.mp4"
  input_path: "work/images/background.png"
  output_path: "work/videos/animated.mp4"
  frames: 14
  fps: 6
  motion: 127
  seed: 44001
  server: "http://192.168.1.64:8100"
  python_path: "{{python_path}}"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `input_path` | Yes | Source image path (PNG) |
| `output_path` | Yes | Output MP4 file path |
| `frames` | No | Number of frames to generate (default: 14) |
| `fps` | No | Output frame rate (default: 6) |
| `motion` | No | Motion intensity 1-255 (default: 127) |
| `seed` | No | Random seed for reproducibility |
| `server` | No | ComfyUI server URL (default: http://192.168.1.64:8100) |
| `python_path` | No | Python interpreter path |

**Motion Guidelines**:

| Motion Value | Effect |
|--------------|--------|
| 1-50 | Subtle movement, gentle parallax |
| 51-127 | Moderate animation, flowing elements |
| 128-180 | Active movement, dynamic scenes |
| 181-255 | High motion, dramatic effects |

**Best Practices**:
- Input images with clear depth cues animate better
- Avoid images with text (will distort)
- Abstract/artistic images often produce interesting results
- Loop the output video to extend duration for narration

## Wan 2.2 (Text-to-Video)

Text-to-video generation using the Wan 2.2 model via ComfyUI. Generates video directly from text prompts.

**Endpoint**: `http://192.168.1.64:6000`

**Models**:
- UNET: `wan2.2_ti2v_5B_fp16.safetensors`
- CLIP: `umt5_xxl_fp8_e4m3fn_scaled.safetensors`
- VAE: `wan2.2_vae.safetensors`

**Step Type**: `text_to_video`

```yaml
- id: vid_intro
  kind: text_to_video
  resume_output: "work/videos/intro.mp4"
  prompt: "Digital workflow visualization, glowing blue data streams flowing through geometric nodes"
  output_path: "work/videos/intro.mp4"
  orientation: "landscape"
  length: 49
  steps: 20
  cfg: 5.0
  seed: 45001
  server: "http://192.168.1.64:6000"
  python_path: "{{python_path}}"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `prompt` | Yes | Text description of desired video |
| `output_path` | Yes | Output MP4 file path |
| `orientation` | No | `portrait`, `landscape`, `square` (default: landscape) |
| `length` | No | Number of frames (default: 81 = ~5s at 16fps) |
| `steps` | No | Sampling steps (default: 20) |
| `cfg` | No | CFG scale (default: 5.0) |
| `seed` | No | Random seed for reproducibility |
| `server` | No | ComfyUI server URL (default: http://192.168.1.64:6000) |
| `python_path` | No | Python interpreter path |

**Output Resolutions** (latent → output):

| Orientation | Latent | Output | Aspect Ratio |
|-------------|--------|--------|--------------|
| landscape | 832x480 | 1664x960 | ~16:9 |
| portrait | 480x832 | 960x1664 | ~9:16 |
| square | 640x640 | 1280x1280 | 1:1 |

**Generation Time**:
- ~13 minutes for 81 frames on RTX 4090
- ~7 minutes for 49 frames

**Prompt Tips**:
- Describe motion: "flowing", "moving", "transforming"
- Include camera: "smooth camera movement", "static shot", "slow zoom"
- Avoid rapid scene changes (model prefers continuous motion)

## midi-cli-rs (Music Generation)

Local MIDI-based music generation using FluidSynth for audio synthesis.

**Binary**: `/path/to/midi-cli-rs/target/release/midi-cli-rs`

**SoundFont**: `VintageDreamsWaves-v2.sf2`

**Step Type**: `run_command`

```yaml
- id: music_intro
  kind: run_command
  resume_output: "assets/music-intro.wav"
  program: "{{midi_cli}}"
  args:
    - "preset"
    - "--mood"
    - "upbeat"
    - "--duration"
    - "12"
    - "--seed"
    - "2026"
    - "--soundfont"
    - "{{soundfont}}"
    - "-o"
    - "assets/music-intro.wav"
```

| Argument | Description |
|----------|-------------|
| `preset` | Use preset generation mode |
| `--mood` | Music mood: `upbeat`, `calm`, `dramatic`, `mysterious` |
| `--duration` | Duration in seconds |
| `--seed` | Random seed for reproducibility |
| `--soundfont` | Path to .sf2 soundfont file |
| `-o` | Output WAV file path |

**Available Moods**:

| Mood | Character |
|------|-----------|
| upbeat | Energetic, positive, driving rhythm |
| calm | Relaxed, ambient, gentle |
| dramatic | Intense, building tension |
| mysterious | Ethereal, uncertain, atmospheric |

## whisper.cpp (Transcription)

Local speech-to-text transcription using whisper.cpp. Used for verifying TTS output and generating subtitles.

**Binary**: `/opt/homebrew/bin/whisper-cli`

**Model**: `ggml-base.en.bin` (English, ~142MB)

**Step Type**: `whisper_transcribe`

```yaml
- id: transcribe_intro
  kind: whisper_transcribe
  resume_output: "output/transcripts/intro.txt"
  input_path: "work/audio/intro.wav"
  output_path: "output/transcripts/intro.txt"
  whisper_cli: "{{whisper_cli}}"
  model: "{{whisper_model}}"
  language: "en"
  format: "txt"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `input_path` | Yes | Path to input audio file (WAV, MP3, etc.) |
| `output_path` | Yes | Path to output transcript file |
| `whisper_cli` | No | Path to whisper-cli binary (default: /opt/homebrew/bin/whisper-cli) |
| `model` | No | Path to whisper model file (default: ~/.whisper-models/ggml-base.en.bin) |
| `language` | No | Language code (default: "en") |
| `format` | No | Output format: "txt", "srt", "vtt" (default: "txt") |

**Available Models**:

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| ggml-tiny.en.bin | 75MB | Fastest | Lower |
| ggml-base.en.bin | 142MB | Fast | Good |
| ggml-small.en.bin | 466MB | Medium | Better |
| ggml-medium.en.bin | 1.5GB | Slow | Best |

**Use Cases**:
- Verify TTS output matches input script
- Generate subtitles/captions (SRT/VTT format)
- Create searchable transcripts
- Quality control for audio content

## normalize_volume (Audio Processing)

Local audio normalization step that adjusts clip volumes to target dB levels.

**Dependencies**: ffmpeg (with volumedetect filter)

**Step Type**: `normalize_volume`

```yaml
# Normalize narration to -25 dB
- id: normalize_intro
  kind: normalize_volume
  clip_path: "work/clips/intro.mp4"
  target_db: -25

# Normalize music to -32 dB (7 dB quieter)
- id: normalize_title
  kind: normalize_volume
  clip_path: "work/clips/title.mp4"
  target_db: -32
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `clip_path` | Yes | Path to clip to normalize (modified in place) |
| `target_db` | No | Target mean volume in dB (default: -25) |

**Standard Volume Levels**:

| Audio Type | Target | Description |
|------------|--------|-------------|
| Narration/Speech | -25 dB | Primary spoken content |
| Background Music | -32 dB | 7 dB quieter than speech |

**Why Post-Process Normalization?**

Applying volume adjustment during clip creation (via ffmpeg -af) often produces inconsistent results due to AAC encoding variations. The proven approach is:

1. Create clips with native audio levels
2. Run `normalize_volume` on each clip (post-processing)
3. Then concatenate the normalized clips

This ensures consistent audio levels across all clips in the final video.

## Ollama (LLM Text Generation)

Local LLM inference using Ollama. Used for generating scripts, prompts, and other text content.

**Server**: `http://localhost:11434`

**Model**: `qwen2.5-coder:14b` (recommended for creative/technical tasks)

**Step Type**: `llm_generate`

```yaml
# First, write the request prompt
- id: write_prompt_request
  kind: write_file
  path: "work/llm-prompts/image-request.txt"
  content: |
    Generate a creative image prompt for a tech visualization.
    Theme: Workflow automation
    Style: Dark, cinematic, cyan accents

# Then, call the LLM
- id: llm_generate_prompt
  kind: llm_generate
  resume_output: "work/llm-outputs/image-prompt.txt"
  system: "You are a creative prompt engineer. Output only the prompt, no explanations."
  user_prompt_path: "work/llm-prompts/image-request.txt"
  output_path: "work/llm-outputs/image-prompt.txt"
  provider: "ollama"
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `system` | Yes | System prompt defining the LLM's role |
| `user_prompt_path` | Yes | Path to file containing user prompt |
| `output_path` | Yes | Path for LLM response output |
| `provider` | Yes | LLM provider (use "ollama") |

**CLI Usage**:

```bash
# Run workflow with Ollama LLM
vwf run workflow.yaml --workdir . --llm-model qwen2.5-coder:14b

# Available models (check with: ollama list)
# - qwen2.5-coder:14b (9GB, best for creative/technical)
# - gemma2:9b (5.4GB, good general purpose)
# - llama3.1:8b (4.9GB, fast and capable)
```

**Use Cases**:
- Generate image prompts for text-to-image
- Write narration scripts
- Create video descriptions
- Generate metadata and titles

## Network Configuration

All GPU services run on a local network server (`192.168.1.64`, hostname: `curiosity`).

```
┌─────────────────────────────────────────────────────────┐
│  GPU Server (192.168.1.64 / curiosity)                  │
│  ├── :8570  FLUX.1 schnell (text-to-image)              │
│  ├── :8100  SVD-XT (image-to-video)                     │
│  ├── :6000  Wan 2.2 (text-to-video)                     │
│  └── :7860  VoxCPM (voice cloning TTS)                  │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Local (workflow host)                                  │
│  ├── :11434  Ollama (LLM text generation)               │
│  ├── midi-cli-rs (music generation)                     │
│  ├── whisper-cli (transcription)                        │
│  └── ffmpeg (normalize_volume, video processing)        │
└─────────────────────────────────────────────────────────┘
```

## Checking Service Status

```bash
# Check if a ComfyUI service is responding
curl -s http://192.168.1.64:8570/system_stats | jq .

# Check TTS server
curl -s http://curiosity:7860/api/health

# List queued jobs on a ComfyUI server
curl -s http://192.168.1.64:6000/queue | jq .
```

## Python Environment

All Python-based steps require a virtual environment with dependencies:

```bash
# Create venv in project directory
cd projects/myproject
uv venv

# Install required packages
source .venv/bin/activate
uv pip install requests pillow

# Reference in workflow.yaml
vars:
  python_path: "/path/to/project/.venv/bin/python"
```

Required packages:
- `requests` - HTTP client for ComfyUI API
- `pillow` - Image processing (for some steps)
