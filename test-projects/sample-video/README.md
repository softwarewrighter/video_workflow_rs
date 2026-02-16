# Sample Video: "What is a Workflow?"

A 30-45 second explainer video produced entirely by VWF to validate the end-to-end workflow.

## Topic

Explain what a workflow is using simple visuals and narration. Target audience: developers new to automation.

## Structure

1. **Hook** (5s): "Ever repeat the same steps over and over?"
2. **Problem** (10s): Manual processes are error-prone and tedious
3. **Solution** (15s): A workflow automates steps in sequence
4. **Benefit** (10s): Save time, reduce errors, focus on creative work
5. **CTA** (5s): "Try VWF - workflows as code"

## Directory Structure

```
sample-video/
|-- assets/           # Input assets (background, music)
|-- work/             # Intermediate files
|   |-- scripts/      # Per-slide narration scripts
|   |-- audio/        # Generated TTS audio
|   |-- images/       # Generated or processed images
|   |-- clips/        # Video clips before assembly
|-- output/           # Final video output
|-- workflow.yaml     # Main workflow definition
```

## Running

```bash
# From repo root
cargo run -p vwf-cli --manifest-path components/vwf-apps/Cargo.toml -- \
  run test-projects/sample-video/workflow.yaml \
  --workdir test-projects/sample-video \
  --allow ffmpeg --allow edge-tts
```

## Phases

1. Generate narration script (LLM)
2. Generate TTS audio (edge-tts)
3. Create simple visuals (placeholder images)
4. Assemble video (ffmpeg)
