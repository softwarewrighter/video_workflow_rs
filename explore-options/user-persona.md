# User Persona

## Primary User: Solo Video Creator

**Name:** Mike (Developer-Creator)

**Role:** Solo developer creating educational video content for YouTube

**Technical Profile:**
- Expert Rust/Python developer
- Comfortable with CLI tools, ffmpeg, shell scripting
- Uses AI coding assistants (Claude Code, etc.)
- Runs local ML models (TTS, lip-sync) on home GPU cluster

**Content Types:**
1. **YouTube Shorts** (5MLC series) - 1080x1920, 2-3 minutes, ML concept education
2. **Explainer Videos** - 1920x1080, 5-20 minutes, software project walkthroughs

**Pain Points:**
- AI agents forget steps, hallucinate parameters, drift from instructions
- Many manual handoffs between tools (ffmpeg, TTS, Playwright, vid-* tools)
- Sequential constraints (TTS can't parallelize) mixed with parallelizable work
- Iterative refinement: want previews before all assets are ready
- Context switching: creating assets (backgrounds, music) while workflow progresses

**Goals:**
- Start a video project with minimal inputs
- See progress and previews continuously
- Parallelize what can be parallelized
- Never lose work due to agent drift
- Review and approve at natural checkpoints

**Workflow Style:**
- Prefers to provide minimal required inputs upfront
- Creates optional assets (backgrounds, music, OBS recordings) in parallel
- Wants to see "best available" preview at any time
- Reviews text content before TTS generation
- Reviews TTS output before final assembly

## Secondary User: AI Coding Agent

**Role:** Implements and extends the framework

**Capabilities:**
- Can read/write files
- Can execute shell commands
- Can call external tools (ffmpeg, vid-*, TTS client)
- Can use Playwright for screenshots

**Constraints:**
- Limited context window
- Needs explicit, focused instructions per step
- Should not improvise parameters
- Must follow deterministic workflows

**Needs from VWF:**
- Clear step definitions with explicit inputs/outputs
- Validation that catches errors early
- Provenance tracking for debugging
- Ability to resume after interruption
