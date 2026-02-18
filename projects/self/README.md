# VWF Demo: Meta Test Video

A YouTube Short that explains VWF using VWF itself. This serves as:
1. Integration test for the workflow engine
2. Demonstration of placeholder handling
3. Real, meaningful output (not nonsense)

## Concept: "5 Things About VWF"

A Shorts-style video covering 5 key concepts about the Video Workflow Framework:

1. **Workflows as Data** - YAML defines steps, not agent memory
2. **Dependency DAG** - Tasks run when inputs are ready
3. **Placeholders** - Work progresses with missing assets
4. **Checkpoints** - Human review at critical points
5. **Incremental Builds** - Only re-run what changed

## Test Scenarios

### Scenario A: All Placeholders
- No background image (solid #1a1a2e)
- No intro/outro (skipped)
- No music (silent)
- Generated narration text
- Mock TTS (silent audio with correct duration)

Expected: Produces viewable video with colored slides and text

### Scenario B: Partial Assets
- Background image provided
- No intro/outro
- Music provided
- Real TTS generation

Expected: Better quality video, still missing intro/outro

### Scenario C: Complete
- All assets provided
- Full TTS
- Intro and outro

Expected: Production-ready video

## Files

```
vwf-demo/
+-- assets/
|   +-- background.jpg      # Optional
|   +-- intro.mp4           # Optional
|   +-- outro.mp4           # Optional
|   +-- music.mp3           # Optional
+-- showcase/
|   +-- slides/             # Generated HTML slides
+-- work/
|   +-- description.txt     # YouTube description
|   +-- narration.txt       # Full narration
|   +-- slides-outline.txt  # Slide content
|   +-- scripts/            # Per-sentence TTS scripts
|   +-- audio/              # Generated audio
|   +-- images/             # Screenshot PNGs
|   +-- clips/              # Video clips
|   +-- preview.mp4         # Current best preview
+-- README.md
```
