# VWF Step Types Reference

This document describes all available step types in the Video Workflow Framework.
Each step type has specific parameters and produces specific outputs.

## Core Step Types

### ensure_dirs
Creates directories if they don't exist.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| dirs | yes | array[string] | List of directory paths to create |

**Example:**
```yaml
- id: create_directories
  kind: ensure_dirs
  dirs:
    - "work/scripts"
    - "work/audio"
    - "work/videos"
    - "output"
```

### write_file
Writes text content to a file.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| path | yes | string | Output file path |
| content | yes | string | Text content to write |

**Example:**
```yaml
- id: write_intro_script
  kind: write_file
  path: "work/scripts/intro.txt"
  content: |
    Welcome to this video about workflow automation.
    Let me show you how it works.
```

### run_command
Executes a shell command. Requires --allow flag.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| program | yes | string | Program to execute |
| args | no | array[string] | Command arguments |
| cwd | no | string | Working directory (relative to workdir) |

**Example:**
```yaml
- id: generate_music
  kind: run_command
  program: midi-cli-rs
  args:
    - "--mood"
    - "upbeat"
    - "--seed"
    - "2026"
    - "--output"
    - "work/audio/background.wav"
```

**Note:** Must run with `--allow midi-cli-rs` flag.

---

## AI Generation Steps

### llm_generate
Generates text using a local LLM (via Ollama).

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| system_prompt | yes | string | System instructions for the LLM |
| user_prompt | yes | string | User prompt/request |
| output_path | yes | string | Output file for generated text |

**Example:**
```yaml
- id: generate_image_prompt
  kind: llm_generate
  resume_output: "work/prompts/hero.txt"
  system_prompt: |
    You are an expert at writing prompts for FLUX.1 image generation.
    Write concise, descriptive prompts that produce high-quality images.
  user_prompt: |
    Write a FLUX.1 prompt for: A professional hero image showing
    a developer working with AI tools, modern office setting.
  output_path: "work/prompts/hero.txt"
```

**Note:** Requires `--llm-model <model>` flag (e.g., `--llm-model qwen2.5-coder:14b`).

### tts_generate
Generates speech audio using voice cloning (VoxCPM).

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| script_path | yes | string | Path to text file with narration |
| output_path | yes | string | Output WAV file path |
| reference_audio | yes | string | Voice clone reference WAV (20-60s) |
| reference_text | yes | string | Transcript of reference audio |
| server | no | string | VoxCPM server URL (default: http://curiosity:7860) |

**Example:**
```yaml
- id: narrate_intro
  kind: tts_generate
  resume_output: "work/audio/intro.wav"
  script_path: "work/scripts/intro.txt"
  output_path: "work/audio/intro.wav"
  reference_audio: "assets/voice-reference.wav"
  reference_text: "This is a sample of my voice for cloning purposes."
```

### text_to_image
Generates images from text prompts (FLUX.1 schnell).

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| prompt | yes | string | Image generation prompt |
| output_path | yes | string | Output PNG file path |
| orientation | no | string | portrait/landscape/square (default: portrait) |
| width | no | integer | Custom width (overrides orientation) |
| height | no | integer | Custom height (overrides orientation) |
| seed | no | integer | Random seed for reproducibility |
| server | no | string | ComfyUI server URL |

**Example:**
```yaml
- id: generate_hero_image
  kind: text_to_image
  resume_output: "work/images/hero.png"
  prompt: "Professional developer at modern desk, coding on multiple monitors, warm lighting, photorealistic"
  output_path: "work/images/hero.png"
  orientation: landscape
  seed: 2026
```

### image_to_video
Animates a still image into video (SVD-XT).

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| input_path | yes | string | Input image path |
| output_path | yes | string | Output MP4 file path |
| frames | no | integer | Number of frames (default: 14) |
| fps | no | integer | Frame rate (default: 6) |
| motion | no | integer | Motion intensity 1-1023 (default: 127) |
| seed | no | integer | Random seed |
| server | no | string | ComfyUI server URL |

**Example:**
```yaml
- id: animate_hero
  kind: image_to_video
  resume_output: "work/videos/hero.mp4"
  input_path: "work/images/hero.png"
  output_path: "work/videos/hero.mp4"
  frames: 14
  motion: 80
```

### text_to_video
Generates video from text prompt (Wan 2.2).

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| prompt | yes | string | Video generation prompt |
| output_path | yes | string | Output MP4 file path |
| orientation | no | string | portrait/landscape/square (default: landscape) |
| length | no | integer | Frame count (default: 81 = ~5s at 16fps) |
| steps | no | integer | Sampling steps (default: 20) |
| cfg | no | float | CFG scale (default: 5.0) |
| seed | no | integer | Random seed |
| server | no | string | ComfyUI server URL |

**Example:**
```yaml
- id: generate_title_video
  kind: text_to_video
  resume_output: "work/videos/title.mp4"
  prompt: "A friendly cartoon dog eating from a bowl labeled 'Dogfood', tech office background"
  output_path: "work/videos/title.mp4"
  orientation: landscape
  length: 81
  seed: 2026
```

---

## Audio Processing Steps

### normalize_volume
Normalizes audio volume to target dB level.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| clip_path | yes | string | Video/audio file to normalize (modified in place) |
| target_db | no | integer | Target mean volume in dB (default: -25) |

**Example:**
```yaml
- id: normalize_narration
  kind: normalize_volume
  clip_path: "work/clips/intro.mp4"
  target_db: -25  # Standard narration level
```

**Volume Standards:**
- Narration/Speech: -25 dB mean
- Background Music: -32 dB mean (7 dB quieter than narration)

### audio_mix
Mixes audio overlay onto a video clip.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| base_clip | yes | string | Video with existing audio (e.g., narration) |
| overlay_audio | yes | string | Audio to overlay (e.g., music) |
| output_path | yes | string | Output video path |
| overlay_volume | no | integer | Overlay volume in dB (default: -32) |
| loop_overlay | no | boolean | Loop overlay to match video length (default: true) |
| fade_out | no | float | Fade out duration in seconds (default: 2.0) |

**Example:**
```yaml
- id: add_background_music
  kind: audio_mix
  base_clip: "work/clips/intro.mp4"
  overlay_audio: "assets/music/background.wav"
  output_path: "work/clips/intro-mixed.mp4"
  overlay_volume: -32
  fade_out: 3.0
```

### whisper_transcribe
Transcribes audio using Whisper.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| input_path | yes | string | Audio/video file to transcribe |
| output_path | yes | string | Output transcript file |
| model | no | string | Whisper model (default: base) |
| language | no | string | Language code (default: auto-detect) |

**Example:**
```yaml
- id: transcribe_narration
  kind: whisper_transcribe
  resume_output: "work/transcripts/intro.txt"
  input_path: "work/audio/intro.wav"
  output_path: "work/transcripts/intro.txt"
```

---

## Video Assembly Steps

### video_concat
Concatenates multiple video clips.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| clips | yes | array[string] | List of video paths (in order) |
| output_path | yes | string | Output video path |
| reencode | no | boolean | Re-encode for mismatched codecs (default: false) |

**Example:**
```yaml
- id: assemble_final
  kind: video_concat
  clips:
    - "work/clips/title.mp4"
    - "work/clips/intro.mp4"
    - "work/clips/body.mp4"
    - "work/clips/outro.mp4"
  output_path: "output/final.mp4"
  reencode: true
```

### create_slide
Generates text slides for titles/bullets.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| text | yes | string | Main text content |
| output_path | yes | string | Output image path |
| template | no | string | title/bullet/quote (default: title) |
| subtext | no | string | Subtitle or attribution |
| background | no | string | Background image path |
| background_color | no | string | Solid background color (default: #1a1a2e) |
| text_color | no | string | Text color (default: #ffffff) |
| orientation | no | string | landscape/portrait (default: landscape) |
| font | no | string | Font name (default: DejaVu-Sans-Bold) |

**Example:**
```yaml
- id: title_slide
  kind: create_slide
  resume_output: "work/images/title.png"
  template: title
  text: "Video Workflow Framework"
  subtext: "Dogfooding Demo"
  output_path: "work/images/title.png"
  orientation: landscape
```

---

## Utility Steps

### split_sections
Splits a text file into sections by delimiter.

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| input_path | yes | string | Input text file |
| output_dir | yes | string | Output directory for section files |
| delimiter | no | string | Section delimiter (default: ---) |
| prefix | no | string | Output file prefix (default: section) |

**Example:**
```yaml
- id: split_script
  kind: split_sections
  input_path: "work/full_script.txt"
  output_dir: "work/scripts"
  delimiter: "---"
  prefix: "segment"
```

---

## Quality Assurance Steps

### llm_audit
Audits produced assets (images/videos) using vision-capable LLMs.

This step uses a recursive approach to validate video/image quality:
1. For videos, extract frames at regular intervals
2. Send each frame to a vision model (llava, qwen2.5vl, llama3.2-vision)
3. Aggregate feedback and report issues
4. Optionally fail the workflow if critical issues found

**Parameters:**
| Name | Required | Type | Description |
|------|----------|------|-------------|
| assets | yes | array[string] | List of image/video paths to audit |
| audit_prompt | yes | string | Instructions for the vision model |
| output_path | yes | string | Path for JSON audit report |
| model | no | string | Vision model name (default: llava) |
| server | no | string | Ollama server URL (default: http://localhost:11434) |
| frame_count | no | integer | Frames to extract from videos (default: 5) |
| fail_on_issues | no | boolean | Fail workflow if critical issues found (default: false) |
| critical_keywords | no | array[string] | Words indicating critical issues |

**Default Critical Keywords:**
- error, broken, corrupt, missing, blank, black screen

**Example:**
```yaml
- id: audit_generated_videos
  kind: llm_audit
  assets:
    - "work/videos/title.mp4"
    - "work/videos/intro.mp4"
    - "work/images/hero.png"
  audit_prompt: |
    Analyze this frame for quality issues:
    - Is the image clear and not corrupted?
    - Are there any visual artifacts or glitches?
    - Does the content match what would be expected for a professional video?
    Describe any issues found. Say "OK" if the frame looks good.
  output_path: "work/reports/audit.json"
  model: llava
  frame_count: 5
  fail_on_issues: false
```

**Supported Vision Models:**
- llava (default, fast)
- qwen2.5vl:7b (more detailed)
- llama3.2-vision:11b (highest quality)

**Output Format:**
```json
{
  "model": "llava",
  "total_assets": 3,
  "total_frames_analyzed": 11,
  "findings": [
    {
      "asset": "work/videos/title.mp4",
      "frame": "/tmp/vwf_audit_123/title_001.png",
      "feedback": "The frame shows a clear animated scene...",
      "is_critical": false
    }
  ],
  "critical_issues": []
}
```

---

## Common Patterns

### Resume/Idempotency
Use `resume_output` to skip steps when output exists (with `--resume` flag):

```yaml
- id: expensive_generation
  kind: text_to_image
  resume_output: "work/images/hero.png"  # Skip if this file exists
  prompt: "..."
  output_path: "work/images/hero.png"
```

### Template Variables
Use `{{var_name}}` for variable substitution:

```yaml
vars:
  project_name: "My Video"
  voice_ref: "assets/my-voice.wav"

steps:
  - id: narrate
    kind: tts_generate
    reference_audio: "{{voice_ref}}"
    # ...
```

### Segments (Semantic Grouping)
Group steps semantically with `segments` to prevent audio conflicts:

```yaml
segments:
  - id: intro
    segment_type: music_only       # No narration allowed
    description: "Title card with background music"
    steps: [title_slide, title_video, intro_music]

  - id: main_content
    segment_type: narration_only   # No music allowed
    steps: [tts_hook, clip_hook, tts_solution, clip_solution]

  - id: outro
    segment_type: music_only
    steps: [outro_slide, outro_video, outro_music]
```

**Segment Types:**
| Type | Description |
|------|-------------|
| music_only | Background music, no narration |
| narration_only | Spoken content, no music |
| mixed | Both allowed (default) |

Segments are optional but help document intent and can be validated.

### Dependencies (DAG Execution)
Use `depends_on` to declare step dependencies:

```yaml
steps:
  - id: setup_dirs
    kind: ensure_dirs
    dirs: ["work/audio", "work/videos"]

  - id: write_script
    kind: write_file
    depends_on: [setup_dirs]
    path: "work/scripts/intro.txt"
    content: "Welcome..."

  - id: tts_intro
    kind: tts_generate
    depends_on: [write_script]
    script_path: "work/scripts/intro.txt"
    # ...

  - id: generate_image
    kind: text_to_image
    depends_on: [setup_dirs]  # Can run in parallel with tts_intro
    # ...
```

**DAG Execution Features:**
- Steps run when all dependencies are satisfied
- Failed steps don't block unrelated work
- Blocked steps are clearly reported with reasons
- Cycle detection runs before execution

**Status Types:**
| Status | Meaning |
|--------|---------|
| OK | Step completed successfully |
| Skipped | Output exists and --resume was used |
| Failed | Step execution error |
| Blocked | Dependency failed, step not attempted |
