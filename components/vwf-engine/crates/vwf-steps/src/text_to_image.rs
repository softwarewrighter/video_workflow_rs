//! Handler for text_to_image step kind.
//!
//! Generates images using FLUX.1 schnell via ComfyUI REST API.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Text prompt for image generation
    prompt: String,
    /// Output image path (.png)
    output_path: String,
    /// Image orientation: portrait (768x1344), landscape (1344x768), square (1024x1024)
    #[serde(default = "default_orientation")]
    orientation: String,
    /// Custom width (overrides orientation)
    width: Option<u32>,
    /// Custom height (overrides orientation)
    height: Option<u32>,
    /// Random seed for reproducibility
    seed: Option<u64>,
    /// ComfyUI server URL
    #[serde(default = "default_server")]
    server: String,
    /// Python interpreter path (default: python3)
    #[serde(default = "default_python")]
    python_path: Option<String>,
}

fn default_python() -> Option<String> {
    None
}

fn default_orientation() -> String {
    "portrait".to_string()
}

fn default_server() -> String {
    "http://192.168.1.64:8570".to_string()
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode text_to_image"))?;

    let prompt = ctx.render(&p.prompt)?;
    let output_path = ctx.render(&p.output_path)?;
    let server = ctx.render(&p.server)?;
    let orientation = ctx.render(&p.orientation)?;

    // Determine dimensions - exact aspect ratios, divisible by 8 for FLUX latent space
    // 720x1280 is standard HD, exact 9:16/16:9, scales cleanly to 1080x1920/1920x1080
    let (width, height) = match (p.width, p.height) {
        (Some(w), Some(h)) => (w, h),
        _ => match orientation.as_str() {
            "landscape" => (1280, 720),  // 16:9 - scales 1.5x to 1920x1080
            "square" => (1024, 1024),
            _ => (720, 1280),  // 9:16 portrait - scales 1.5x to 1080x1920
        },
    };

    let seed = p.seed.unwrap_or_else(|| rand::random());
    let python = p.python_path
        .map(|pp| ctx.render(&pp))
        .transpose()?
        .unwrap_or_else(|| "python3".to_string());

    // Generate image via Python script
    let script = image_gen_script(&server, &prompt, width, height, seed, &output_path);

    let status = Command::new(&python)
        .args(["-c", &script])
        .status()
        .with_context(|| ctx.error_context("spawn text_to_image python"))?;

    if !status.success() {
        anyhow::bail!("Image generation failed with exit code: {:?}", status.code());
    }

    Ok(())
}

fn image_gen_script(server: &str, prompt: &str, width: u32, height: u32, seed: u64, output: &str) -> String {
    // Escape prompt for Python string
    let prompt_escaped = prompt.replace('\\', "\\\\").replace('"', "\\\"");

    format!(r#"
import requests
import time
from pathlib import Path

SERVER = "{server}"
PROMPT = "{prompt_escaped}"
WIDTH = {width}
HEIGHT = {height}
SEED = {seed}
OUTPUT = "{output}"

workflow = {{
    "prompt": {{
        "1": {{"class_type": "CheckpointLoaderSimple",
              "inputs": {{"ckpt_name": "flux1-schnell-fp8.safetensors"}}}},
        "2": {{"class_type": "CLIPTextEncode",
              "inputs": {{"text": PROMPT, "clip": ["1", 1]}}}},
        "3": {{"class_type": "CLIPTextEncode",
              "inputs": {{"text": "", "clip": ["1", 1]}}}},
        "4": {{"class_type": "EmptyLatentImage",
              "inputs": {{"width": WIDTH, "height": HEIGHT, "batch_size": 1}}}},
        "5": {{"class_type": "KSampler",
              "inputs": {{"model": ["1", 0], "positive": ["2", 0],
                         "negative": ["3", 0], "latent_image": ["4", 0],
                         "seed": SEED, "steps": 4, "cfg": 1.0,
                         "sampler_name": "euler", "scheduler": "simple",
                         "denoise": 1.0}}}},
        "6": {{"class_type": "VAEDecode",
              "inputs": {{"samples": ["5", 0], "vae": ["1", 2]}}}},
        "7": {{"class_type": "SaveImage",
              "inputs": {{"images": ["6", 0], "filename_prefix": "vwf"}}}},
    }}
}}

# Submit
print(f"Submitting to {{SERVER}}...")
r = requests.post(f"{{SERVER}}/prompt", json=workflow)
r.raise_for_status()
prompt_id = r.json()["prompt_id"]
print(f"Job ID: {{prompt_id}}")

# Poll for completion
print("Waiting for generation...", end="", flush=True)
while True:
    r = requests.get(f"{{SERVER}}/history/{{prompt_id}}")
    data = r.json()
    if prompt_id in data and data[prompt_id].get("status", {{}}).get("completed"):
        break
    print(".", end="", flush=True)
    time.sleep(2)
print(" done")

# Extract filename
outputs = data[prompt_id]["outputs"]
for node_out in outputs.values():
    if "images" in node_out:
        filename = node_out["images"][0]["filename"]
        break

# Download
print(f"Downloading: {{filename}}")
r = requests.get(f"{{SERVER}}/view", params={{"filename": filename, "subfolder": "", "type": "output"}})
r.raise_for_status()

Path(OUTPUT).parent.mkdir(parents=True, exist_ok=True)
with open(OUTPUT, "wb") as f:
    f.write(r.content)
print(f"Saved: {{OUTPUT}}")
"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_image_script() {
        let script = image_gen_script(
            "http://localhost:8570",
            "A test prompt",
            720,
            1280,
            42,
            "/tmp/out.png",
        );
        assert!(script.contains("flux1-schnell-fp8.safetensors"));
        assert!(script.contains("A test prompt"));
        assert!(script.contains("720"));
        assert!(script.contains("1280"));
    }
}
