#!/bin/bash
# Common variables for sample-video scripts

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# VoxCPM TTS settings
TTS_SERVER="http://curiosity:7860"
REF_WAV="/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav"
REF_TEXT="In this session, I'm going to write a small command line tool and explain the decision making process as I go. I'll begin with a basic skeleton, argument parsing, a configuration loader, and a minimal main function. Once everything compiles, I'll run it with a few sample inputs to confirm the behavior. After that, I'll be fine the internal design. I'll reorganize the functions, extract shared logic, and add error messages that actually help the user understand what went wrong. None of this is complicated, but it's the kind of work that separates a rough prototype from a tool someone can rely on. As we move forward, I'll highlight why I chose certain patterns, some decisions, optimize clarity, while others optimize performance or extensibility. The important thing is to understand the trade-offs well enough that the code feels intentional instead of accidental."

# Video tools
VID_SLIDE="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-slide"
VID_IMAGE="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-image"
VID_CONCAT="/Users/mike/github/softwarewrighter/video-publishing/tools/target/release/vid-concat"

# Whisper model
WHISPER_MODEL="/Users/mike/.whisper-models/ggml-base.en.bin"

# Directories
SCRIPTS_DIR="$PROJECT_DIR/work/scripts"
AUDIO_DIR="$PROJECT_DIR/work/audio"
SLIDES_DIR="$PROJECT_DIR/work/slides"
CLIPS_DIR="$PROJECT_DIR/work/clips"
OUTPUT_DIR="$PROJECT_DIR/output"
