//! Handler for tts_generate step kind.
//!
//! Generates TTS audio using VoxCPM voice cloning via Gradio API.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Path to script text file
    script_path: String,
    /// Output audio path (.wav)
    output_path: String,
    /// VoxCPM server URL (default: http://curiosity:7860)
    #[serde(default = "default_server")]
    server: String,
    /// Path to reference voice WAV
    reference_audio: String,
    /// Transcript of reference audio
    reference_text: String,
    /// Python interpreter path (default: python3)
    #[serde(default = "default_python")]
    python_path: String,
}

fn default_server() -> String {
    "http://curiosity:7860".to_string()
}

fn default_python() -> String {
    "python3".to_string()
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode tts_generate"))?;

    let script_path = ctx.render(&p.script_path)?;
    let output_path = ctx.render(&p.output_path)?;
    let server = ctx.render(&p.server)?;
    let ref_audio = ctx.render(&p.reference_audio)?;
    let ref_text = ctx.render(&p.reference_text)?;

    // Read script text
    let script_text = ctx.rt.read_text(&script_path)?;
    let python_path = ctx.render(&p.python_path)?;

    // Make output path absolute (relative paths are relative to workdir)
    let abs_output = if output_path.starts_with('/') {
        output_path.clone()
    } else {
        ctx.rt.workdir().join(&output_path).to_string_lossy().to_string()
    };

    // Call TTS via Python gradio_client
    let status = Command::new(&python_path)
        .args(["-c", &tts_script(&server, &ref_audio, &ref_text, &script_text, &abs_output)])
        .status()
        .with_context(|| ctx.error_context("spawn tts python"))?;

    if !status.success() {
        anyhow::bail!("TTS generation failed with exit code: {:?}", status.code());
    }

    Ok(())
}

fn tts_script(server: &str, ref_audio: &str, ref_text: &str, text: &str, output: &str) -> String {
    format!(r#"
from gradio_client import Client, handle_file
from pathlib import Path
import shutil

client = Client("{server}")
result = client.predict(
    text_input="""{text}""",
    prompt_wav_path_input=handle_file("{ref_audio}"),
    prompt_text_input="""{ref_text}""",
    cfg_value_input=2.0,
    inference_timesteps_input=15,
    do_normalize=False,
    api_name="/generate"
)
Path("{output}").parent.mkdir(parents=True, exist_ok=True)
shutil.copy(result, "{output}")
print(f"Generated: {output}")
"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_tts_script() {
        let script = tts_script("http://localhost:7860", "/ref.wav", "hello", "world", "/out.wav");
        assert!(script.contains("Client(\"http://localhost:7860\")"));
        assert!(script.contains("text_input=\"\"\"world\"\"\""));
    }
}
