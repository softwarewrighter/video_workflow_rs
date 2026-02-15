# Explainer Video Creation Workflow

This document describes the step-by-step process for creating explainer videos using the deterministic video rendering pipeline.

## Inputs Needed

### From the User

1. **Source Project** - GitHub repo or local project to explain
2. **Key Features** - What aspects to highlight (CLI, Web UI, API, etc.)
3. **Demo Recordings** - OBS screen recordings or VHS terminal demos
4. **Music Selection** - Track from `reference/music/` for title and epilog
5. **Branding** - Title background image, project name styling
6. **Hashtags** - Channel signature tags: `#VibeCoding #PersonalSoftware #MLStudy` plus topic-specific

### Standard Assets (Already Available)

- **Voice Reference**: `/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav` (63s)
- **Avatar**: `/Users/mike/github/softwarewrighter/video-publishing/reference/curmudgeon.mp4`
- **Epilog Frame**: Shared "Like & Subscribe" frame from prior projects
- **Music Library**: `reference/music/` folder

---

## Project Structure

```
projects/<project-name>/
├── assets/
│   ├── svg/           # SVG slides (00-title through 99b-epilog)
│   ├── images/        # Title background, screenshots
│   └── music/         # Project-specific music (optional)
├── work/
│   ├── scripts/       # Narration text files (*.txt)
│   ├── audio/         # Generated TTS audio (*.wav)
│   ├── clips/         # Video clips (OBS, VHS, composited)
│   ├── stills/        # PNG renders from SVG
│   ├── avatar/        # Avatar processing files
│   ├── preview/       # Preview assembly workspace
│   └── generate-tts.sh  # TTS generation script
├── tts/
│   ├── .venv/         # Python venv for gradio_client
│   └── client.py      # VoxCPM client
├── README.md          # Project documentation
└── NEXT-STEPS.md      # Progress tracking
```

---

## Workflow Steps

### Phase 1: Project Setup

```bash
mkdir -p projects/<name>/{assets/{svg,images},work/{scripts,audio,clips,stills,avatar,preview},tts}
```

1. Create project directory structure
2. Copy `tts/client.py` from a prior project
3. Set up Python venv: `cd tts && uv venv && source .venv/bin/activate && uv pip install gradio_client`
4. Create `work/generate-tts.sh` with correct reference file and prompt text

### Phase 2: Content Planning

1. **Outline segments** - List what to show (intro, demos, features, CTA)
2. **Write narration scripts** - One `*.txt` file per segment in `work/scripts/`
3. **Plan visuals** - Which segments need SVG slides vs. screen recordings

**Narration Rules:**
- Maximum 320 characters per script
- Periods and commas only (no dashes, colons, question marks)
- Spell out numbers as words ("thirty thousand" not "30,000")
- Spell out symbols ("percent" not "%", "dollar" not "$")
- Spell out acronyms with spaces ("M L" not "ML")

### Phase 3: Create SVG Slides

Reference `/docs/style-guide.md` for exact values.

**Font Sizes:**
| Element | Size |
|---------|------|
| Headlines | 96px |
| Subtitles | 48px |
| Body text | 36px |
| Labels | 32px |
| Stroke widths | 5px |

**Color Palette (Dracula):**
- Background: `#1a1a2e`
- Cyan: `#00d4ff`
- Red: `#ff6b6b`
- Green: `#4ade80`
- Yellow: `#ffd93d`

**Avatar Safe Zone:** Keep bottom-right clear (x=1400-1920, y=800-1080)

Validate with: `/style-check`

### Phase 4: Generate TTS Audio

```bash
cd work && ./generate-tts.sh
```

The script:
1. Scans `work/scripts/*.txt` for narration files
2. Generates audio to `work/audio/` using VoxCPM
3. Skips files that already have audio
4. Verifies output with whisper

**CRITICAL:** TTS calls must be sequential, never parallel.

**Verify each audio:**
```bash
ffmpeg -y -i work/audio/01-hook.wav -ar 16000 -ac 1 -c:a pcm_s16le /tmp/verify.wav
whisper-cli -m ~/.whisper-models/ggml-base.en.bin -f /tmp/verify.wav -nt
```

**Fixing run-on sentences:** Use punctuation (periods, commas) for natural pauses. If insufficient, add extra punctuation in the script.

### Phase 5: Record Demos

**VHS Terminal Recordings:**
```tape
Set Shell "bash"
Set FontSize 32
Set Width 1920
Set Height 1080
Set Theme "Dracula"
Set TypingSpeed 50ms
Set Padding 20
```

**OBS Screen Recordings:**
- Record at native resolution
- Will be scaled to 1920x1080 during processing
- Note timestamps for segment boundaries

### Phase 6: Render SVG → PNG → Video

```bash
# SVG → PNG
rsvg-convert -w 1920 -h 1080 assets/svg/01-hook.svg -o work/stills/01-hook.png

# PNG → Base video (duration = audio + 1s)
AUDIO_DUR=$(ffprobe -v error -show_entries format=duration -of csv=p=0 work/audio/01-hook.wav)
VID_DUR=$(echo "$AUDIO_DUR + 1" | bc)
$VID_IMAGE --image work/stills/01-hook.png --duration $VID_DUR --output work/clips/01-hook-base.mp4
```

### Phase 7: Combine Video + Audio

```bash
ffmpeg -y -i work/clips/01-hook-base.mp4 -i work/audio/01-hook.wav \
  -filter_complex "[1:a]apad=pad_dur=0.2[a]" \
  -map 0:v -map "[a]" \
  -c:v copy -c:a aac -b:a 192k \
  work/clips/01-hook.mp4
```

**Always add 200ms padding** at end of narration to prevent run-on between clips.

### Phase 8: Process OBS/VHS Recordings

**NEVER truncate recordings.** If video is longer than audio, pad audio:

```bash
ffmpeg -y -i source.mp4 -i audio.wav \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2" \
  -filter_complex "[1:a]apad=whole_dur=VIDEO_DUR[a]" \
  -map 0:v -map "[a]" -c:v libx264 -crf 18 -c:a aac -b:a 192k \
  output.mp4
```

### Phase 9: Add Avatar (Optional)

```bash
# Stretch avatar to match audio duration
$VID_AVATAR --avatar ../video-publishing/reference/curmudgeon.mp4 \
  --duration $AUDIO_DUR --output work/avatar/01-hook-stretched.mp4

# Remove audio from stretched avatar
ffmpeg -y -i work/avatar/01-hook-stretched.mp4 -an -c:v copy work/avatar/01-hook-silent.mp4

# Lip-sync
$VID_LIPSYNC --avatar work/avatar/01-hook-silent.mp4 \
  --audio work/audio/01-hook.wav \
  --output work/avatar/01-hook-lipsync.mp4 \
  --server http://hive:3015

# Composite onto base video
$VID_COMPOSITE --content work/clips/01-hook-base.mp4 \
  --avatar work/avatar/01-hook-lipsync.mp4 \
  --output work/clips/01-hook-composited.mp4 --size 200
```

### Phase 10: Normalize All Clips

```bash
./scripts/normalize-volume.sh work/clips/*.mp4
```

Every clip must be 44100Hz stereo at -25dB before concatenation.

### Phase 11: Create Title and Epilog

**Title (7s with music):**
```bash
ffmpeg -y -loop 1 -i work/stills/00-title-composited.png \
  -i "$MUSIC" \
  -filter_complex "[1:a]atrim=0:7,volume=0.5[a]" \
  -map 0:v -map "[a]" \
  -c:v libx264 -crf 18 -t 7 -r 30 \
  -c:a aac -b:a 192k -pix_fmt yuv420p \
  work/clips/00-title.mp4
```

**Epilog (12s with music):**
- Use shared `epilog-frame.png` ("Like & Subscribe")
- Same music as title, no fade

**Epilog Extension (7s with music fade):**
```bash
ffmpeg -y -loop 1 -i work/stills/epilog-frame.png \
  -i "$MUSIC" \
  -filter_complex "[1:a]atrim=12:19,asetpts=PTS-STARTPTS,afade=t=out:st=4:d=3,volume=0.5[a]" \
  -map 0:v -map "[a]" \
  -c:v libx264 -crf 18 -t 7 -r 30 \
  -c:a aac -b:a 192k -pix_fmt yuv420p \
  work/clips/99c-epilog-ext.mp4
```

### Phase 12: Concatenate Final Video

**Create concat list** (absolute paths, one per line):
```
/full/path/to/clips/00-title.mp4
/full/path/to/clips/01-hook.mp4
...
/full/path/to/clips/99-cta.mp4
/full/path/to/clips/99b-epilog.mp4
/full/path/to/clips/99c-epilog-ext.mp4
```

**Concatenate:**
```bash
$VID_CONCAT --list work/preview/complete-concat.txt --output work/preview-complete.mp4 --reencode
./scripts/normalize-volume.sh work/preview-complete.mp4
```

### Phase 13: Create Thumbnail

Derive from title with larger text (2x height, 1.2x width) and translucent shim for readability.

### Phase 14: Write Description

Create `work/description.txt` with:
- Summary paragraph
- Features list
- Links (live demo, source code)
- Chapters with timestamps
- Music credit
- Hashtags: `#VibeCoding #PersonalSoftware #MLStudy` + topic-specific

---

## Video Tools

Located in `~/github/softwarewrighter/video-publishing/tools/target/release/`:

| Tool | Purpose |
|------|---------|
| `vid-image` | Create video from image |
| `vid-avatar` | Stretch avatar to duration |
| `vid-lipsync` | Lip-sync avatar to audio |
| `vid-composite` | Overlay avatar on content |
| `vid-concat` | Concatenate clips (ALWAYS use this, not raw ffmpeg) |

---

## Common Issues & Fixes

| Issue | Cause | Fix |
|-------|-------|-----|
| Silent audio (-91dB) | Two-step audio combination | Use `vid-image --audio` or explicit `-map 0:v -map 1:a` |
| Audio sample rate mismatch | Mixed 44100/48000Hz | Run `normalize-volume.sh` on EVERY clip |
| Wrong dimensions | OBS Retina recording | Scale to 1920x1080 with ffmpeg |
| Truncated demo | Used `-shortest` flag | Pad audio with `apad=whole_dur=VIDEO_DUR` |
| Run-on narration | No pause between clips | Add `apad=pad_dur=0.2` to each clip |
| Run-on sentences | TTS no pause between sentences | Use punctuation in script |
| Garbled TTS | Wrong reference file or prompt | Use 63s reference with matching prompt text |

---

## Typical Video Structure

| # | Segment | Type | Duration |
|---|---------|------|----------|
| 00 | Title | Music | 7s |
| 01 | Hook | Narration | 15-20s |
| 02-N | Content slides | Narration | varies |
| N+1 | Demo intro | Narration | 10-15s |
| N+2 | Demo | Narration over recording | varies |
| 99 | CTA | Narration | 10-15s |
| 99b | Epilog | Music | 12s |
| 99c | Epilog extension | Music fade | 7s |

Total typical duration: 5-8 minutes
