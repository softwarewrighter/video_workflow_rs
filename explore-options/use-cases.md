# Use Cases

## UC-1: Start New 5MLC Episode

**Trigger:** User provides episode number and 5 ML concepts

**Minimal Inputs:**
- Episode number (e.g., 17)
- 5 concept names

**Optional Inputs (can be added later):**
- Background image
- Music track
- Intro video
- Outro video
- Custom narration text
- Custom ELI5 analogies

**System Behavior:**
1. Generate project structure
2. Generate description.txt, narration.txt, slides-outline.txt
3. CHECKPOINT: User reviews text files
4. Generate HTML slides (with placeholder background if none provided)
5. Capture slide screenshots
6. Split narration into TTS scripts
7. Generate TTS audio (sequential, GPU-bound)
8. Verify TTS with whisper
9. Combine audio segments
10. Create video clips (slides + audio)
11. If intro/outro provided: scale and add music
12. If intro/outro missing: generate placeholder or skip
13. Concatenate all available clips
14. Normalize audio
15. Output: work/preview.mp4 (always current best)

**Progressive Output:**
- After step 4: Can preview HTML slides in browser
- After step 8: Can listen to TTS audio
- After step 10: Can preview individual video clips
- After step 13: Can preview full video (with placeholders where needed)

---

## UC-2: Start New Explainer Video

**Trigger:** User provides source project (git repo or local path)

**Minimal Inputs:**
- Project path or URL
- Key features to highlight

**Optional Inputs:**
- Demo recordings (OBS, VHS)
- Background image
- Music track
- Custom narration

**System Behavior:**
1. Analyze project (README, code structure)
2. Generate outline: hook, sections, demos, CTA
3. Generate narration scripts
4. CHECKPOINT: User reviews text files
5. Create SVG slides
6. CHECKPOINT: User reviews slide layout (via /style-check)
7. Render SVG -> PNG
8. Generate TTS audio
9. Create base video clips
10. If demo recordings provided: process and composite
11. If avatar requested: stretch, lip-sync, composite
12. Add title/epilog with music
13. Concatenate and normalize
14. Generate thumbnail
15. Write description with chapters

---

## UC-3: Add Missing Asset Mid-Workflow

**Trigger:** User provides background image after workflow started

**System Behavior:**
1. Detect new asset (background.jpg added to assets/)
2. Identify affected steps (HTML slides, screenshots, video clips)
3. Re-run only affected steps (not TTS, not text generation)
4. Update preview.mp4 with new background
5. Notify user: "Background applied, preview updated"

---

## UC-4: Review and Approve TTS

**Trigger:** TTS generation complete

**System Behavior:**
1. Generate individual audio review clips
2. Generate combined audio for each slide
3. CHECKPOINT: User listens and approves
4. If user requests redo:
   - User edits script file
   - System regenerates only that segment
   - Returns to checkpoint

---

## UC-5: Preview Current State

**Trigger:** User requests preview at any time

**System Behavior:**
1. Identify all completed artifacts
2. For missing artifacts, use:
   - Solid color for missing backgrounds
   - Silent audio for missing TTS
   - Static image for missing video
   - Skip missing intro/outro
3. Concatenate available + placeholder clips
4. Output: work/preview-current.mp4
5. Show report: "Preview includes 7/12 segments, missing: intro, 02-demo, outro"

---

## UC-6: Resume After Interruption

**Trigger:** User returns to in-progress project

**System Behavior:**
1. Read work/state.json (or run.json)
2. Identify completed steps (have outputs, outputs unchanged)
3. Identify invalidated steps (inputs changed since output)
4. Identify blocked steps (waiting for prerequisites)
5. Show status summary
6. Resume from first incomplete step
7. Parallelize where safe

---

## UC-7: Parallel Work on Multiple Segments

**Trigger:** User has GPU available, wants max throughput

**System Behavior:**
1. Identify parallelizable work:
   - SVG -> PNG rendering (CPU-bound, parallelizable)
   - ffmpeg encoding (CPU-bound, parallelizable)
   - Screenshot capture (I/O-bound, parallelizable)
2. Identify sequential work:
   - TTS generation (GPU-bound, must be sequential)
   - Whisper verification (GPU-bound, must be sequential)
3. Run parallel work while TTS runs sequentially
4. Update preview as clips complete

---

## UC-8: Generate Both Short and Explainer

**Trigger:** User wants both formats from same source

**Minimal Inputs:**
- Source project
- Key features

**System Behavior:**
1. Analyze project once (shared)
2. Generate two outlines (short format, explainer format)
3. CHECKPOINT: User reviews both
4. Fork into parallel workflows:
   - Short: 1080x1920, 2-3 min, 5 concepts
   - Explainer: 1920x1080, 5-20 min, full walkthrough
5. Share assets where possible (TTS voice, background, music)
6. Output: work/short/final.mp4, work/explainer/final.mp4
