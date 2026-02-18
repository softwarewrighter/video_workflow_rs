# VWF Workflow Structure

This document describes the YAML schema for VWF workflow files.

## Basic Structure

```yaml
version: 1
name: "My Video Workflow"
description: "Optional description of what this workflow produces"

vars:
  project_name: "My Project"
  voice_ref: "assets/voice.wav"

segments:
  - id: intro
    segment_type: music_only
    steps: [title_slide, title_video]
  - id: main
    segment_type: narration_only
    steps: [script_intro, tts_intro, vid_intro]

steps:
  - id: step_id
    kind: step_type
    # step-specific parameters
```

## Top-Level Fields

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| version | yes | integer | Schema version (always 1) |
| name | yes | string | Workflow name |
| description | no | string | Human-readable description |
| vars | no | map | Template variables |
| segments | no | array | Semantic step groupings |
| steps | yes | array | Ordered list of workflow steps |

---

## Variables (vars)

Variables enable reuse and configuration:

```yaml
vars:
  project_name: "Demo Video"
  tts_server: "http://curiosity:7860"
  voice_ref: "assets/my-voice.wav"
  voice_text: "This is my voice reference text."
  output_dir: "output"
```

**Usage in steps:**
```yaml
- id: narrate
  kind: tts_generate
  server: "{{tts_server}}"
  reference_audio: "{{voice_ref}}"
  reference_text: "{{voice_text}}"
  output_path: "{{output_dir}}/narration.wav"
```

**Variable Naming:**
- Use snake_case
- Be descriptive: `tts_server` not `server`
- Group related vars: `voice_ref`, `voice_text`

---

## Segments

Segments provide semantic grouping to prevent invalid audio combinations:

```yaml
segments:
  - id: intro
    segment_type: music_only
    description: "Title card with background music, no narration"
    steps:
      - create_title
      - gen_title_video
      - add_intro_music

  - id: hook
    segment_type: narration_only
    description: "Opening hook with narration, no music"
    steps:
      - write_hook
      - tts_hook
      - create_hook_video

  - id: content
    segment_type: mixed
    description: "Main content with both narration and quiet background music"
    steps:
      - write_main
      - tts_main
      - mix_music
```

**Segment Types:**

| Type | Audio Content | Use Case |
|------|---------------|----------|
| music_only | Music, no narration | Intro/outro cards |
| narration_only | Narration, no music | Main content |
| mixed | Both allowed | Content with background music |

**Validation:**
- All step IDs in segments must exist in steps array
- Segment IDs must be unique
- Steps can only belong to one segment

---

## Steps

Each step has these common fields:

```yaml
- id: unique_step_id        # Required: unique identifier
  kind: step_type           # Required: one of the step types
  resume_output: "path"     # Optional: skip if this file exists
  # ... step-specific fields
```

### Step ID Conventions

- Use descriptive names: `tts_intro` not `step1`
- Include action: `write_`, `gen_`, `mix_`, `normalize_`
- Include target: `_intro`, `_hook`, `_final`

Good examples:
- `write_intro_script`
- `tts_intro`
- `gen_hero_image`
- `normalize_intro_audio`
- `concat_final`

### Resume Output

Use `resume_output` to enable incremental builds:

```yaml
- id: expensive_step
  kind: text_to_video
  resume_output: "work/videos/title.mp4"  # Skip if exists
  # ...
```

Run with `--resume` flag to skip completed steps.

---

## Step Ordering

Steps execute in declaration order. Dependencies are implicit based on file paths:

```yaml
steps:
  # 1. Create directories first
  - id: setup
    kind: ensure_dirs
    dirs: ["work/scripts", "work/audio", "work/videos"]

  # 2. Write content
  - id: write_intro
    kind: write_file
    path: "work/scripts/intro.txt"
    content: "..."

  # 3. Generate audio (depends on script)
  - id: tts_intro
    kind: tts_generate
    script_path: "work/scripts/intro.txt"  # Uses output from write_intro
    output_path: "work/audio/intro.wav"

  # 4. Final assembly (depends on all previous)
  - id: concat
    kind: video_concat
    clips:
      - "work/videos/intro.mp4"
      - "work/videos/main.mp4"
```

---

## Complete Example

```yaml
version: 1
name: "Product Explainer"
description: "60-second explainer video for SaaS product"

vars:
  project_name: "MyApp"
  voice_ref: "assets/voice-sample.wav"
  voice_text: "This is a sample of the narrator's voice."
  tts_server: "http://curiosity:7860"
  flux_server: "http://192.168.1.64:8570"

segments:
  - id: intro
    segment_type: music_only
    steps: [gen_title, title_to_video]
  - id: hook
    segment_type: narration_only
    steps: [write_hook, tts_hook, gen_hook_visual, hook_video]
  - id: demo
    segment_type: narration_only
    steps: [write_demo, tts_demo, screen_capture]
  - id: outro
    segment_type: music_only
    steps: [gen_cta, cta_video]

steps:
  # Setup
  - id: setup_dirs
    kind: ensure_dirs
    dirs:
      - "work/scripts"
      - "work/audio"
      - "work/images"
      - "work/videos"
      - "output"

  # Intro segment
  - id: gen_title
    kind: create_slide
    resume_output: "work/images/title.png"
    template: title
    text: "{{project_name}}"
    subtext: "Simplify Your Workflow"
    output_path: "work/images/title.png"

  - id: title_to_video
    kind: image_to_video
    resume_output: "work/videos/title.mp4"
    input_path: "work/images/title.png"
    output_path: "work/videos/title.mp4"
    motion: 40

  # Hook segment
  - id: write_hook
    kind: write_file
    path: "work/scripts/hook.txt"
    content: |
      What if you could automate your entire workflow
      with just a few lines of configuration?

  - id: tts_hook
    kind: tts_generate
    resume_output: "work/audio/hook.wav"
    script_path: "work/scripts/hook.txt"
    output_path: "work/audio/hook.wav"
    server: "{{tts_server}}"
    reference_audio: "{{voice_ref}}"
    reference_text: "{{voice_text}}"

  - id: gen_hook_visual
    kind: text_to_image
    resume_output: "work/images/hook.png"
    prompt: "Modern office worker looking frustrated at complex software interface"
    output_path: "work/images/hook.png"
    orientation: landscape
    server: "{{flux_server}}"

  - id: hook_video
    kind: run_command
    program: ffmpeg
    args:
      - "-y"
      - "-loop"
      - "1"
      - "-i"
      - "work/images/hook.png"
      - "-i"
      - "work/audio/hook.wav"
      - "-c:v"
      - "libx264"
      - "-tune"
      - "stillimage"
      - "-c:a"
      - "aac"
      - "-shortest"
      - "work/videos/hook.mp4"

  # ... more steps ...

  # Final assembly
  - id: concat_final
    kind: video_concat
    clips:
      - "work/videos/title.mp4"
      - "work/videos/hook.mp4"
      - "work/videos/demo.mp4"
      - "work/videos/cta.mp4"
    output_path: "output/{{project_name}}_explainer.mp4"
    reencode: true
```

---

## Running Workflows

```bash
# Dry run (preview without executing)
vwf run workflow.yaml --workdir project --dry-run

# Full execution
vwf run workflow.yaml --workdir project --allow ffmpeg --allow midi-cli-rs

# With LLM model
vwf run workflow.yaml --workdir project --llm-model qwen2.5-coder:14b

# Resume (skip completed steps)
vwf run workflow.yaml --workdir project --resume

# With variable overrides
vwf run workflow.yaml --workdir project --var project_name="New Name"
```
