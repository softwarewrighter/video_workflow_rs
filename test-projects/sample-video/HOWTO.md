# How This Video Was Generated

This document describes the step-by-step process used to create the "What is a Workflow?" sample video.

## Prerequisites

1. **VoxCPM TTS Server** running at `http://curiosity:7860`
2. **Voice reference**: `mike-medium-ref-1.wav` (63 seconds)
3. **Python venv** with `gradio_client` installed
4. **Video tools**: vid-slide, vid-image, vid-concat
5. **Whisper** for verification

## Pipeline Overview

```
Scripts (.txt) --> TTS (VoxCPM) --> Audio (.wav) --> Whisper Verify
                                         |
Slide Images (.png) <-- vid-slide        |
         |                               |
         v                               v
    vid-image (image + audio) --> Clips (.mp4)
                                         |
                                         v
                                  vid-concat --> preview.mp4
```

## Step 1: Generate Scripts (VWF Workflow)

The workflow YAML generates script sections using mock LLM:

```bash
cd test-projects/sample-video
cargo run -p vwf-cli -- run workflow.yaml --workdir .
```

Output: `work/scripts/{01-hook,02-problem,03-solution,04-benefit,05-cta}.txt`

## Step 2: Generate TTS Audio

Using VoxCPM voice cloning with the 63s reference:

```bash
./scripts/generate-tts.sh
```

Or manually:
```bash
source .venv/bin/activate
python tts_client.py \
    --server http://curiosity:7860 \
    --reference /path/to/mike-medium-ref-1.wav \
    --prompt-text "In this session, I'm going to write..." \
    --text "$(cat work/scripts/01-hook.txt)" \
    --output work/audio/01-hook.wav
```

Output: `work/audio/*.wav`

## Step 3: Verify with Whisper

Check that TTS pronunciation matches the script:

```bash
./scripts/verify-tts.sh
```

Common fixes needed:
- "error-prone" -> "prone to errors" (avoid TTS mispronunciation)
- "run it" -> "runn it" (phonetic spelling)
- "missed step" -> "missed, step" (add pause)

## Step 4: Build Video

Generate slides and combine with audio:

```bash
./scripts/build-video.sh
```

This:
1. Creates slide images with vid-slide
2. Combines each slide + audio with vid-image
3. Concatenates all clips with vid-concat

Output: `output/preview.mp4`

## Resume After Interruption

If the workflow is interrupted (power outage, etc.), use `--resume`:

```bash
cargo run -p vwf-cli -- run workflow.yaml --workdir . --resume
```

This skips steps whose `output_path` already exists and is valid.

## File Structure

```
test-projects/sample-video/
├── workflow.yaml          # VWF workflow definition
├── tts_client.py          # VoxCPM Gradio client
├── scripts/
│   ├── common.sh          # Shared variables
│   ├── generate-tts.sh    # TTS generation
│   ├── verify-tts.sh      # Whisper verification
│   └── build-video.sh     # Video assembly
├── work/
│   ├── scripts/           # Narration text (from workflow)
│   ├── audio/             # TTS audio (from generate-tts.sh)
│   ├── slides/            # Slide images (from build-video.sh)
│   └── clips/             # Video segments (from build-video.sh)
└── output/
    └── preview.mp4        # Final video
```

## Timing

| Section | Duration |
|---------|----------|
| 01-hook | 4.2 sec |
| 02-problem | 7.4 sec |
| 03-solution | 7.5 sec |
| 04-benefit | 7.0 sec |
| 05-cta | 5.8 sec |
| **Total** | **~32 sec** |
