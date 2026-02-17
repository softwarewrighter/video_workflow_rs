#!/bin/bash
# Common variables for tt-rs-explainer project
# Landscape format: 1920x1080 (16:9)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Video dimensions (16:9 landscape)
WIDTH=1920
HEIGHT=1080

# VoxCPM TTS settings
TTS_SERVER="http://curiosity:7860"
REF_WAV="/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav"
REF_TEXT="In this session, I'm going to write a small command line tool and explain the decision making process as I go. I'll begin with a basic skeleton, argument parsing, a configuration loader, and a minimal main function. Once everything compiles, I'll run it with a few sample inputs to confirm the behavior. After that, I'll be fine the internal design. I'll reorganize the functions, extract shared logic, and add error messages that actually help the user understand what went wrong. None of this is complicated, but it's the kind of work that separates a rough prototype from a tool someone can rely on. As we move forward, I'll highlight why I chose certain patterns, some decisions, optimize clarity, while others optimize performance or extensibility. The important thing is to understand the trade-offs well enough that the code feels intentional instead of accidental."

# Video tools
VID_IMAGE="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-image"
VID_CONCAT="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-concat"
VID_SLIDE="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-slide"

# Whisper model
WHISPER_MODEL="/Users/mike/.whisper-models/ggml-base.en.bin"

# Directories
ASSETS_DIR="$PROJECT_DIR/assets"
SCRIPTS_DIR="$PROJECT_DIR/work/scripts"
AUDIO_DIR="$PROJECT_DIR/work/audio"
IMAGES_DIR="$PROJECT_DIR/work/images"
CLIPS_DIR="$PROJECT_DIR/work/clips"
OUTPUT_DIR="$PROJECT_DIR/output"

# Section durations (in seconds)
TITLE_DURATION=5
PLACEHOLDER_DURATION=5
EPILOG_EXT_DURATION=7

# Colors (SoftwareWrighter branding)
BG_COLOR="#ffffff"
TEXT_COLOR="#333333"
ACCENT_COLOR="#00d4ff"
