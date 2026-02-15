# YouTube Shorts Production Workflow

This document describes the step-by-step process for creating YouTube Shorts videos, specifically for the "5 ML Concepts" (5MLC) series.

## Inputs Required From User

Before starting an episode, I need:

### Required
1. **Episode number** - Which episode (e.g., 5MLC-17)
2. **5 concepts** - The ML concepts to cover (from `docs/chatgpt-schedule-episode-12-through-23.txt`)
3. **Intro video** - MP4 file for the intro animation (e.g., `seventeen-skywriter.mp4`)
4. **Outro video** - MP4 file for the outro animation
5. **Background image** - JPG for slide backgrounds (e.g., `waterfall-splashes.jpg`)
6. **Music track** - MP3 from `video-publishing/reference/music/` (e.g., `Sly Sky*.mp3`)

### Optional (has defaults)
- Custom narration text (otherwise I generate from concept definitions)
- Custom ELI5 analogies (otherwise I generate them)

## Directory Structure Created

```
projects/5MLC-NN/
+-- assets/
|   +-- intro.mp4        # Copied from user-provided file
|   +-- outro.mp4        # Copied from user-provided file
|   +-- background.jpg   # Copied from user-provided file
|   +-- music.mp3        # Copied from reference/music/
+-- showcase/
|   +-- images/
|   |   +-- background.jpg
|   +-- slides/
|       +-- 00-intro.html
|       +-- 01-concept-name.html
|       +-- 02-concept-name.html
|       +-- 03-concept-name.html
|       +-- 04-concept-name.html
|       +-- 05-concept-name.html
|       +-- 06-cta.html
+-- tts/
|   +-- client.py        # VoxCPM Gradio client
|   +-- .venv/           # Python environment
+-- work/
    +-- scripts/         # Individual sentence TTS scripts
    +-- audio/           # Generated WAV files
    +-- images/          # Screenshot PNGs of slides
    +-- clips/           # Intermediate video clips
    +-- description.txt  # YouTube description
    +-- narration.txt    # Full narration script
    +-- slides-outline.txt
    +-- generate-tts.sh
    +-- combine-audio.sh
    +-- build-video.sh
    +-- final.mp4        # Final output
```

## Workflow Steps

### Phase 1: Content Preparation

1. **Create work/*.txt files for user review**
   - `description.txt` - YouTube description with Blog/Discord links
   - `narration.txt` - Full narration organized by slide
   - `slides-outline.txt` - Slide content with categories, descriptions, ELI5s

2. **User reviews and edits** the txt files in their editor

3. **Copy assets** to `assets/` directory:
   ```bash
   cp <intro-video> projects/5MLC-NN/assets/intro.mp4
   cp <outro-video> projects/5MLC-NN/assets/outro.mp4
   cp <background> projects/5MLC-NN/assets/background.jpg
   cp <music> projects/5MLC-NN/assets/music.mp3
   ```

### Phase 2: TTS Generation

1. **Create TTS scripts** - Split narration into individual sentences:
   ```
   work/scripts/00-intro-1.txt
   work/scripts/00-intro-2.txt
   work/scripts/01-concept-name-1.txt
   ...
   ```
   Each file contains ONE sentence (max ~200 chars).

2. **Set up TTS environment**:
   ```bash
   cd projects/5MLC-NN/tts
   cp ../../../projects/5MLC-15/tts/client.py .  # Copy from prior project
   uv venv && source .venv/bin/activate
   uv pip install gradio_client
   ```

3. **Create generate-tts.sh** with:
   - Reference audio: `/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav`
   - Prompt text: Must EXACTLY match the reference audio transcript (63 seconds)
   - VoxCPM server: `http://curiosity:7860`
   - Whisper verification after each file

4. **Run TTS generation** (SEQUENTIAL, never parallel):
   ```bash
   ./generate-tts.sh
   ```
   This generates individual WAV files and verifies each with whisper-cli.

5. **Create combine-audio.sh** to:
   - Add 0.2s padding between sentences
   - Combine sentences into per-slide audio
   - Add 1s padding at start, 0.5s at end of each slide
   ```bash
   ./combine-audio.sh
   ```

### Phase 3: Slide Creation

1. **Create HTML slides** in `showcase/slides/`:
   - 00-intro.html - Episode number, title, concept list
   - 01-05 concept slides - Category, name, description, ELI5
   - 06-cta.html - Checklist of concepts, follow CTA

2. **Cache-busting** - CRITICAL to prevent stale screenshots:
   ```html
   <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate">
   ```
   And for background images:
   ```css
   background: url('../images/background.jpg?ts=1771125236423') ...
   ```
   Generate timestamp: `python3 -c "import time; print(int(time.time() * 1000))"`

3. **Copy background to showcase**:
   ```bash
   cp assets/background.jpg showcase/images/
   ```

### Phase 4: Slide Capture

1. **Start local server**:
   ```bash
   cd showcase && python3 -m http.server 5078 &
   ```

2. **Capture screenshots** using Playwright MCP at 1080x1920:
   - Navigate to each slide URL
   - Take screenshot, save to `work/images/`
   - Close browser

3. **Rename screenshots** to remove timestamps:
   ```bash
   cd work/images
   for f in *-2026-*.png; do
     newname=$(echo "$f" | sed 's/-2026-[^.]*//');
     mv "$f" "$newname"
   done
   ```

### Phase 5: Video Build

1. **Create build-video.sh** that:
   - Scales intro/outro to 1080x1920 (they may be different sizes)
   - Creates slide clips from images + audio
   - Adds music to intro (volume 0.3, fade out)
   - Adds music to outro (volume 0.3, fade in/out)
   - Concatenates all clips with `vid-concat --reencode`
   - Normalizes audio with `vid-volume --normalize`

2. **Run build**:
   ```bash
   ./build-video.sh
   ```

3. **Preview**:
   ```bash
   open final.mp4
   ```

### Phase 6: Review & Commit

1. **User reviews** the preview video

2. **Fix issues** if needed (regenerate TTS, recapture slides, rebuild)

3. **Commit**:
   ```bash
   git add projects/5MLC-NN/
   git commit -m "Add 5MLC episode NN: [concept list]"
   ```

## Key Scripts

### generate-tts.sh
```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"

TTS_DIR="../tts"
SCRIPTS_DIR="scripts"
AUDIO_DIR="audio"
REF_WAV="/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav"
REF_TEXT="In this session, I'm going to write a small command line tool..."  # Full 63s transcript

mkdir -p "$AUDIO_DIR"
source "$TTS_DIR/.venv/bin/activate"

for script in "$SCRIPTS_DIR"/*.txt; do
    basename=$(basename "$script" .txt)
    output="$AUDIO_DIR/${basename}.wav"
    text=$(cat "$script")

    echo "Generating: $output"
    python "$TTS_DIR/client.py" \
        --server http://curiosity:7860 \
        --reference "$REF_WAV" \
        --prompt-text "$REF_TEXT" \
        --text "$text" \
        --output "$output" \
        --steps 15

    echo "Verifying with whisper..."
    whisper-cli -m ~/.local/share/whisper-cpp/models/ggml-medium.en.bin \
        -f "$output" --no-timestamps 2>/dev/null
done
```

### combine-audio.sh
```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"
AUDIO_DIR="audio"

# For each slide, combine sentences with padding
# Example for 4-sentence concept:
for i in 1 2 3 4; do
  sox $AUDIO_DIR/01-concept-$i.wav $AUDIO_DIR/01-concept-$i-pad.wav pad 0 0.2
done
sox $AUDIO_DIR/01-concept-{1,2,3,4}-pad.wav $AUDIO_DIR/01-concept-combined.wav
sox $AUDIO_DIR/01-concept-combined.wav $AUDIO_DIR/01-concept.wav pad 1 0.5

# Cleanup
rm -f $AUDIO_DIR/*-pad.wav $AUDIO_DIR/*-combined.wav
```

### build-video.sh
```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"

TOOLS="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release"
ASSETS="../assets"

# 1. Create intro with music (scaled to 1080x1920)
ffmpeg -y -i "$ASSETS/intro.mp4" -i "$ASSETS/music.mp3" \
  -filter_complex "[0:v]scale=1080:1920:...[vout];[1:a]volume=0.3...[aout]" \
  -map "[vout]" -map "[aout]" ... clips/00-intro-video.mp4

# 2. Create slide clips (image + audio)
for each segment:
  ffmpeg -y -loop 1 -i images/$NAME.png -t $DUR ... clips/$NAME-silent.mp4
  ffmpeg -y -i clips/$NAME-silent.mp4 -i audio/$NAME.wav ... clips/$NAME-with-audio.mp4

# 3. Create outro with music (scaled to 1080x1920)
# Similar to intro

# 4. Concatenate (MUST use vid-concat --reencode, never raw ffmpeg)
$TOOLS/vid-concat --list clips/concat-list.txt --output clips/combined.mp4 --reencode

# 5. Normalize audio
$TOOLS/vid-volume --input clips/combined.mp4 --output final.mp4 --normalize
```

## Common Issues & Fixes

| Issue | Cause | Fix |
|-------|-------|-----|
| Wrong episode number in video | Browser cached old HTML | Use cache-busting timestamps |
| Garbled TTS audio | Wrong prompt-text | Must EXACTLY match reference audio |
| Silent audio mid-video | Used raw ffmpeg concat | Use `vid-concat --reencode` |
| Intro/outro wrong size | Different source resolution | Scale to 1080x1920 in ffmpeg |
| TTS drops words | Parallel TTS calls | Generate SEQUENTIALLY |

## Typical Durations

- Intro slide: ~12-16s
- Each concept: ~15-25s
- CTA slide: ~10s
- Intro video: ~5s
- Outro video: ~5s
- **Total: ~120-135s**
