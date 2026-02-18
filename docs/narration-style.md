# Narration Script Style Guide

Guidelines for writing narration scripts that produce high-quality TTS audio.

## Acronyms

**Never use raw acronyms in narration scripts.** TTS models struggle with letter sequences and often produce garbled or incorrect audio.

### Options

1. **Spell phonetically** (preferred for brand names):
   - `VWF` → `Vee Double-You Eff`
   - `API` → `A-P-I` or `ay pee eye`
   - `CLI` → `command line` or `C-L-I`
   - `TTS` → `text to speech`
   - `LLM` → `L-L-M` or `large language model`

2. **Expand to full name**:
   - `VWF` → `Video Workflow Framework`
   - `YAML` → `YAML` (already pronounceable as "yammel")
   - `JSON` → `JSON` (pronounceable as "jay-son")

3. **Avoid entirely** - rephrase the sentence:
   - Before: "Use the CLI to run workflows"
   - After: "Use the command line to run workflows"

## Punctuation for Pacing

- Use periods for full pauses
- Use commas for brief pauses
- Use em-dashes for dramatic pauses: "The answer — is automation"
- Avoid semicolons (TTS handles them inconsistently)

## Numbers

- Spell out numbers one through ten
- Use digits for 11 and above
- Spell out numbers at sentence start: "Fifteen steps later..."

## Technical Terms

- Hyphenate compound technical terms for clarity:
  - `text-to-speech` not `text to speech`
  - `pre-built` not `prebuilt`

## Common Substitutions

| Written | Spoken |
|---------|--------|
| VWF | Vee Double-You Eff |
| API | A-P-I |
| CLI | command line |
| TTS | text-to-speech |
| LLM | large language model |
| GPU | G-P-U |
| CPU | C-P-U |
| URL | U-R-L |
| YAML | yaml (one syllable) |
| JSON | jay-son |
| SQL | sequel |
| ffmpeg | eff eff em peg |

## Verification

Always run `scripts/verify-tts.sh` after generating audio to compare Whisper transcription against the original script. Mismatches indicate pronunciation issues.
