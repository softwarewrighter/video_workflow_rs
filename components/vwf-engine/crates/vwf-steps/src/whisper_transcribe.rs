//! Whisper transcription step for audio-to-text conversion.
//!
//! Uses whisper.cpp CLI to transcribe audio files, useful for:
//! - Verifying TTS output matches expected script
//! - Generating subtitles/captions
//! - Creating transcripts from recorded audio
//!
//! # Prerequisites
//!
//! - whisper-cli installed (whisper.cpp)
//! - Model file (e.g., ggml-base.en.bin)

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Path to input audio file (WAV, MP3, etc.)
    input_path: String,
    /// Path to output transcript file (.txt)
    output_path: String,
    /// Path to whisper model file (.bin)
    #[serde(default = "default_model")]
    model: String,
    /// Path to whisper-cli binary
    #[serde(default = "default_whisper_cli")]
    whisper_cli: String,
    /// Language code (default: "en")
    #[serde(default = "default_language")]
    language: String,
    /// Output format: "txt", "srt", "vtt" (default: "txt")
    #[serde(default = "default_format")]
    format: String,
}

fn default_model() -> String {
    "/Users/mike/.whisper-models/ggml-base.en.bin".to_string()
}

fn default_whisper_cli() -> String {
    "/opt/homebrew/bin/whisper-cli".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_format() -> String {
    "txt".to_string()
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode whisper_transcribe"))?;

    let input_path = ctx.render(&p.input_path)?;
    let output_path = ctx.render(&p.output_path)?;
    let model = ctx.render(&p.model)?;
    let whisper_cli = ctx.render(&p.whisper_cli)?;
    let language = ctx.render(&p.language)?;
    let format = ctx.render(&p.format)?;

    // Resolve paths
    let input_abs = if input_path.starts_with('/') {
        input_path.clone()
    } else {
        ctx.rt.workdir().join(&input_path).to_string_lossy().to_string()
    };

    let output_abs = if output_path.starts_with('/') {
        output_path.clone()
    } else {
        ctx.rt.workdir().join(&output_path).to_string_lossy().to_string()
    };

    // Check input exists
    if !std::path::Path::new(&input_abs).exists() {
        bail!("Input audio not found: {}", input_abs);
    }

    // Check model exists
    if !std::path::Path::new(&model).exists() {
        bail!("Whisper model not found: {}", model);
    }

    println!("Transcribing: {}", input_abs);
    println!("  Model: {}", model);
    println!("  Language: {}", language);

    // Convert to 16kHz mono WAV (whisper requirement)
    let temp_wav = format!("{}.whisper_temp.wav", input_abs);
    let convert_status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &input_abs,
            "-ar", "16000",
            "-ac", "1",
            "-c:a", "pcm_s16le",
            &temp_wav,
        ])
        .output()
        .with_context(|| ctx.error_context("ffmpeg convert for whisper"))?;

    if !convert_status.status.success() {
        let stderr = String::from_utf8_lossy(&convert_status.stderr);
        bail!("Failed to convert audio for whisper: {}", stderr);
    }

    // Run whisper transcription
    let whisper_output = Command::new(&whisper_cli)
        .args([
            "-m", &model,
            "-f", &temp_wav,
            "-l", &language,
            "-nt", // no timestamps for plain text
        ])
        .output()
        .with_context(|| ctx.error_context("whisper transcription"))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_wav);

    if !whisper_output.status.success() {
        let stderr = String::from_utf8_lossy(&whisper_output.stderr);
        bail!("Whisper transcription failed: {}", stderr);
    }

    // Get transcript from stdout
    let transcript = String::from_utf8_lossy(&whisper_output.stdout)
        .trim()
        .to_string();

    // Create output directory if needed
    if let Some(parent) = std::path::Path::new(&output_abs).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Write based on format
    match format.as_str() {
        "txt" => {
            std::fs::write(&output_abs, &transcript)
                .with_context(|| format!("Failed to write transcript: {}", output_abs))?;
        }
        "srt" | "vtt" => {
            // For SRT/VTT, run whisper again with timestamps
            let whisper_output = Command::new(&whisper_cli)
                .args([
                    "-m", &model,
                    "-f", &input_abs,
                    "-l", &language,
                    if format == "srt" { "-osrt" } else { "-ovtt" },
                    "-of", &output_abs.replace(&format!(".{}", format), ""),
                ])
                .output()
                .with_context(|| ctx.error_context("whisper subtitle generation"))?;

            if !whisper_output.status.success() {
                let stderr = String::from_utf8_lossy(&whisper_output.stderr);
                bail!("Whisper subtitle generation failed: {}", stderr);
            }
        }
        _ => bail!("Unknown format: {} (use txt, srt, or vtt)", format),
    }

    println!("  Output: {}", output_abs);
    println!("  Transcript: {}...", &transcript.chars().take(60).collect::<String>());

    Ok(())
}
