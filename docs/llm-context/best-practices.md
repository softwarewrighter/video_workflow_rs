# VWF Best Practices

Guidelines for creating effective video production workflows.

## Audio Standards

### Volume Levels

| Content Type | Target dB | Notes |
|--------------|-----------|-------|
| Narration | -25 dB | Primary audio, clear and present |
| Background Music | -32 dB | 7 dB below narration |
| Sound Effects | -28 dB | Slightly below narration |

### Audio Quality

- **Sample Rate:** 44100 Hz (CD quality)
- **Channels:** Stereo (2 channels)
- **Format:** AAC for video, WAV for intermediates

### Music Placement Rules

1. **Never overlap music with narration at equal volume**
   - Use `audio_mix` with `overlay_volume: -32` for background music
   - Or use `segment_type: music_only` for music-only sections

2. **Fade music at segment boundaries**
   - Use `fade_out: 2.0` (or more) for smooth transitions
   - Music should fade before narration begins

3. **Title/outro segments are music-only**
   - No narration over title cards
   - Music establishes mood

---

## Narration Guidelines

### Script Writing

Per `docs/narration-style.md`:

**Avoid acronyms - spell them out phonetically:**

| Acronym | Say Instead |
|---------|-------------|
| VWF | "Vee Double-You Eff" |
| TTS | "text to speech" |
| TTI | "text to image" |
| CLI | "command line interface" |
| API | "A-P-I" or "application programming interface" |
| URL | "U-R-L" or "web address" |
| YAML | "YAML" (actual pronunciation) |

**Avoid special characters in narration:**
- Don't use `--flags` or `/paths` - describe them verbally
- Spell out symbols: "equals sign" not `=`

**Script length guidelines:**
- Keep individual segments under 60 seconds
- ~150 words per minute for natural pacing
- Break long content into multiple segments

### Voice Reference Quality

For TTS voice cloning:
- **Duration:** 20-60 seconds of clean speech
- **Content:** Natural conversation, not reading
- **Audio:** No background noise, music, or echo
- **Transcript:** Exact transcription of reference audio

---

## Video Structure

### Standard Segment Order

```
1. Title (5s) - music_only
   └── Animated title card, background music

2. Hook (10-15s) - narration_only
   └── Attention-grabbing problem statement

3. Solution (15-20s) - narration_only
   └── Introduce your solution

4. Demo (30-60s) - narration_only or mixed
   └── Show the product/process in action

5. Benefits (15-20s) - narration_only
   └── Key value propositions

6. CTA (10s) - narration_only then music
   └── Call to action, then music fade

7. Outro (5s) - music_only
   └── Logo/branding with music
```

### Video Orientation

| Content Type | Orientation | Resolution |
|--------------|-------------|------------|
| YouTube | landscape | 1920x1080 |
| TikTok/Reels | portrait | 1080x1920 |
| Square promo | square | 1080x1080 |

Set orientation in workflow vars:
```yaml
vars:
  orientation: landscape  # or portrait, square
```

---

## File Organization

### Standard Directory Structure

```
project/
├── assets/
│   ├── voice-reference.wav
│   ├── logo.png
│   └── music/
├── work/
│   ├── scripts/
│   ├── audio/
│   ├── images/
│   └── videos/
├── output/
│   └── final.mp4
└── workflow.yaml
```

### Naming Conventions

**Scripts:** `{segment}-{description}.txt`
- `01-hook.txt`, `02-solution.txt`, `03-demo.txt`

**Audio:** `{segment}.wav`
- `hook.wav`, `solution.wav`

**Images:** `{segment}-{type}.png`
- `hook-background.png`, `demo-screenshot.png`

**Videos:** `{segment}.mp4`
- `title.mp4`, `hook.mp4`, `final.mp4`

---

## Incremental Development

### Use resume_output

Always specify `resume_output` for expensive steps:

```yaml
- id: expensive_generation
  kind: text_to_video
  resume_output: "work/videos/title.mp4"  # Skip if exists
  # ...
```

### Development Workflow

1. **Start with dry run:**
   ```bash
   vwf run workflow.yaml --workdir project --dry-run
   ```

2. **Run incrementally:**
   ```bash
   vwf run workflow.yaml --workdir project --resume
   ```

3. **Force regenerate specific steps:**
   - Delete the `resume_output` file
   - Run again with `--resume`

### Testing Order

1. Test `ensure_dirs` and `write_file` steps first
2. Test TTS with short scripts before full narration
3. Test image generation before video generation
4. Test concatenation last

---

## Service Reliability

### Pre-flight Checks

Before running a workflow:

1. **Check required services are running:**
   ```bash
   curl -s http://curiosity:7860/api/predict  # VoxCPM
   curl -s http://192.168.1.64:8570/system_stats  # FLUX.1
   ```

2. **Verify model availability:**
   ```bash
   ollama list  # Check LLM models
   ```

3. **Test with short content first**

### Error Recovery

If a step fails:

1. Check service health
2. Review step parameters
3. Check disk space and permissions
4. Re-run with `--resume` (will skip completed steps)

### Timeout Considerations

| Step Type | Typical Duration | Notes |
|-----------|------------------|-------|
| tts_generate | 30-60s | Depends on script length |
| text_to_image | 10-30s | FLUX.1 schnell is fast |
| image_to_video | 2-5 min | SVD-XT animation |
| text_to_video | 10-15 min | Wan 2.2 is slow |

Plan workflow execution accordingly. Consider running overnight for long workflows.

---

## Common Mistakes

### 1. Music overlapping narration
**Wrong:**
```yaml
- id: add_music_everywhere
  kind: audio_mix
  base_clip: "work/all_clips.mp4"
  overlay_audio: "music.wav"
  overlay_volume: -20  # Too loud!
```

**Right:**
```yaml
segments:
  - id: intro
    segment_type: music_only
    steps: [title_video]
  - id: content
    segment_type: narration_only
    steps: [main_video]
```

### 2. Not using resume_output
**Wrong:**
```yaml
- id: slow_step
  kind: text_to_video
  output_path: "work/vid.mp4"  # Will regenerate every run!
```

**Right:**
```yaml
- id: slow_step
  kind: text_to_video
  resume_output: "work/vid.mp4"
  output_path: "work/vid.mp4"
```

### 3. Acronyms in narration scripts
**Wrong:**
```yaml
content: "Use the VWF CLI to run TTS generation."
```

**Right:**
```yaml
content: "Use the Vee Double-You Eff command line interface to run text to speech generation."
```

### 4. Missing directory creation
**Wrong:**
```yaml
- id: write_script
  kind: write_file
  path: "work/scripts/intro.txt"  # Directory might not exist!
```

**Right:**
```yaml
- id: setup
  kind: ensure_dirs
  dirs: ["work/scripts"]

- id: write_script
  kind: write_file
  path: "work/scripts/intro.txt"
```

### 5. Hardcoded paths instead of variables
**Wrong:**
```yaml
- id: tts
  kind: tts_generate
  server: "http://curiosity:7860"
  reference_audio: "/home/user/voice.wav"
```

**Right:**
```yaml
vars:
  tts_server: "http://curiosity:7860"
  voice_ref: "assets/voice-reference.wav"

steps:
  - id: tts
    kind: tts_generate
    server: "{{tts_server}}"
    reference_audio: "{{voice_ref}}"
```
