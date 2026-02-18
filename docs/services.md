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

## Network Configuration

All GPU services run on a local network server (`192.168.1.64`). The TTS service runs on `curiosity` (a separate host).

```
┌─────────────────────────────────────────────────────────┐
│  GPU Server (192.168.1.64)                              │
│  ├── :8570  FLUX.1 schnell (text-to-image)              │
│  ├── :8100  SVD-XT (image-to-video)                     │
│  └── :6000  Wan 2.2 (text-to-video)                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  TTS Server (curiosity)                                 │
│  └── :7860  VoxCPM (voice cloning TTS)                  │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Local (workflow host)                                  │
│  └── midi-cli-rs (music generation, no network)         │
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
