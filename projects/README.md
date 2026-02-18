# Video Projects

Work-in-progress video projects using the VWF (Video Workflow Framework).

## Active Projects

| Project | Description | Format | Status |
|---------|-------------|--------|--------|
| **tt-rs** | ToonTalk Rust rewrite explainer (TBT #4) | Landscape 1920x1080 | In Progress |
| **self** | VWF self-explainer (meta video) | Landscape 1920x1080 | Planning |
| **sleepy-coder-2** | sleepy-coder continual learning agent | Landscape 1920x1080 | Planning |

## Demo Projects

| Project | Description | Format |
|---------|-------------|--------|
| **demo-landscape** | Sample landscape video workflow | 1920x1080 |
| **demo-portrait/sample-short** | Sample YouTube Short | Portrait 1080x1920 |
| **demo-portrait/vwf-promo** | VWF promo short | Portrait 1080x1920 |

## Project Structure

Each project follows a standard layout:

```
project/
├── README.md        # Project overview
├── workflow.yaml    # VWF workflow definition
├── tts_client.py    # TTS API client
├── .gitignore       # Output exclusions
├── assets/          # Source media (music, images, fonts)
├── docs/            # Research, outlines, planning
├── scripts/         # Build scripts
│   ├── common.sh    # Shared variables and paths
│   └── build-video.sh
├── work/            # Generated intermediate files
│   ├── audio/       # TTS narration WAVs
│   ├── clips/       # Individual video clips
│   ├── images/      # Rendered slides
│   └── scripts/     # Section narration text
└── output/          # Final renders
    └── preview.mp4
```

## Audio Standards

See `docs/tutorial.md` for audio normalization standards:
- Target narration: -25 dB mean
- Target music: -32 dB mean (7 dB below narration)
- Reference: `epilog.mp4` assets
