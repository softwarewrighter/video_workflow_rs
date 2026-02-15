# Workflow Templates

## Template: 5MLC Episode (Shorts)

```yaml
name: 5mlc-episode
description: YouTube Short for "5 ML Concepts in 5 Minutes" series
version: 1
aspect_ratio: 1080x1920  # Portrait

inputs:
  required:
    episode_number:
      type: integer
      description: Episode number (e.g., 17)

    concepts:
      type: list[string]
      count: 5
      description: Five ML concepts to cover

  optional:
    background_image:
      type: file
      extensions: [jpg, png]
      placeholder: solid_color("#1a1a2e")

    music_track:
      type: file
      extensions: [mp3, wav]
      default: reference/music/default.mp3

    intro_video:
      type: file
      extensions: [mp4]
      placeholder: skip

    outro_video:
      type: file
      extensions: [mp4]
      placeholder: skip

    custom_narration:
      type: file
      extensions: [txt]
      description: Override generated narration

phases:
  - name: content
    tasks: [generate_outline]
    checkpoint: review_text
    outputs: [narration.txt, slides-outline.txt, description.txt]

  - name: slides
    tasks: [create_html_slides, capture_screenshots]
    outputs: [showcase/slides/*.html, work/images/*.png]

  - name: audio
    tasks: [split_scripts, generate_tts, verify_tts, combine_audio]
    checkpoint: review_audio
    constraints:
      tts: sequential
    outputs: [work/audio/*.wav]

  - name: video
    tasks: [create_clips, process_intro, process_outro, concat, normalize]
    outputs: [work/final.mp4]

preview:
  after_phase: [slides, audio, video]
  include_placeholders: true
```

## Template: Explainer Video

```yaml
name: explainer-video
description: Long-form explainer video for software projects
version: 1
aspect_ratio: 1920x1080  # Landscape

inputs:
  required:
    project_path:
      type: path
      description: Git repo or local project path

    key_features:
      type: list[string]
      description: Features to highlight

  optional:
    demo_recordings:
      type: list[file]
      extensions: [mp4]
      description: OBS or VHS recordings

    vhs_tapes:
      type: list[file]
      extensions: [tape]
      description: VHS tape scripts to record

    background_image:
      type: file
      placeholder: solid_color("#1a1a2e")

    music_track:
      type: file
      default: reference/music/default.mp3

    avatar:
      type: file
      extensions: [mp4]
      description: Avatar video for lip-sync
      default: reference/curmudgeon.mp4

    use_avatar:
      type: boolean
      default: false

phases:
  - name: analysis
    tasks: [analyze_project, generate_outline]
    checkpoint: review_outline
    outputs: [work/outline.txt, work/segment-list.txt]

  - name: content
    tasks: [generate_narration, generate_svg_slides]
    checkpoint: review_content
    outputs: [work/scripts/*.txt, assets/svg/*.svg]

  - name: assets
    parallel: true
    tasks:
      - render_svg_to_png
      - record_vhs_demos  # If vhs_tapes provided
      - process_obs_recordings  # If demo_recordings provided
    outputs: [work/stills/*.png, work/clips/*.mp4]

  - name: audio
    tasks: [generate_tts, verify_tts]
    checkpoint: review_audio
    constraints:
      tts: sequential
    outputs: [work/audio/*.wav]

  - name: avatar  # If use_avatar
    condition: inputs.use_avatar
    tasks: [stretch_avatar, lipsync_avatar, composite_avatar]
    constraints:
      lipsync: sequential  # GPU-bound
    outputs: [work/avatar/*.mp4]

  - name: assembly
    tasks: [create_base_clips, add_audio, create_title, create_epilog, concat, normalize]
    outputs: [work/final.mp4]

  - name: metadata
    tasks: [generate_thumbnail, write_description_with_chapters]
    outputs: [work/thumbnail.png, work/description.txt]

preview:
  after_phase: [assets, audio, assembly]
  include_placeholders: true
```

## Template: Dual Output (Short + Explainer)

```yaml
name: dual-video
description: Generate both Short and Explainer from same source
version: 1

inputs:
  required:
    project_path:
      type: path

    key_features:
      type: list[string]

  optional:
    # Shared assets
    background_image:
      type: file
      shared: true

    music_track:
      type: file
      shared: true

    # Short-specific
    short_concepts:
      type: list[string]
      count: 5
      description: Subset of features for Short

    # Explainer-specific
    demo_recordings:
      type: list[file]

outputs:
  short:
    path: work/short/final.mp4
    workflow: 5mlc-episode

  explainer:
    path: work/explainer/final.mp4
    workflow: explainer-video

phases:
  - name: shared_analysis
    tasks: [analyze_project]
    outputs: [work/shared/analysis.json]

  - name: fork
    parallel: true
    branches:
      - workflow: 5mlc-episode
        inputs:
          concepts: $short_concepts
          background_image: $background_image
          music_track: $music_track
        output_dir: work/short/

      - workflow: explainer-video
        inputs:
          project_path: $project_path
          key_features: $key_features
          demo_recordings: $demo_recordings
          background_image: $background_image
          music_track: $music_track
        output_dir: work/explainer/
```

## Template Variables

Templates support variable substitution:

```yaml
# In template definition
tasks:
  generate_tts:
    inputs:
      reference: "{{voice_reference}}"
      prompt_text: "{{voice_prompt}}"

# Default values in template
defaults:
  voice_reference: /Users/mike/.../mike-medium-ref-1.wav
  voice_prompt: "In this session, I'm going to write..."

# User override at runtime
vwf run 5mlc-episode.yaml \
  --var episode_number=17 \
  --var "concepts=[attention,transformer,embedding,loss,gradient]"
```

## Checkpoint Definitions

```yaml
checkpoints:
  review_text:
    message: |
      Review the following files before proceeding:
      - work/narration.txt
      - work/slides-outline.txt
      - work/description.txt

      Edit as needed, then run: vwf approve review_text
    timeout: null  # Wait indefinitely
    on_reject: regenerate  # Or: abort, retry

  review_audio:
    message: |
      Listen to TTS audio in work/audio/
      If any segment needs re-recording, edit the script and run:
        vwf redo tts <segment>
      When satisfied: vwf approve review_audio
    preview: work/audio-preview.mp4  # Combined audio with timings
```

## Constraint Definitions

```yaml
constraints:
  tts:
    type: sequential
    description: TTS calls must run one at a time (GPU memory)

  lipsync:
    type: sequential
    resource: gpu
    description: Lip-sync is GPU-bound

  screenshot:
    type: parallel
    max: 4
    description: Can run multiple browsers

  ffmpeg:
    type: parallel
    max: cpu_count
    description: CPU-bound encoding
```
