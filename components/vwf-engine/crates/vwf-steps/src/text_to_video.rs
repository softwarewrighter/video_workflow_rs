//! Handler for text_to_video step kind.
//!
//! Generates video from a text prompt using Wan 2.2 via ComfyUI REST API.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Text prompt for video generation
    prompt: String,
    /// Output video path (.mp4)
    output_path: String,
    /// Video orientation: portrait, landscape, square (default: landscape)
    #[serde(default = "default_orientation")]
    orientation: String,
    /// Number of frames to generate (default: 81 = ~5s at 16fps)
    #[serde(default = "default_length")]
    length: u32,
    /// Sampling steps (default: 20)
    #[serde(default = "default_steps")]
    steps: u32,
    /// CFG scale (default: 5.0)
    #[serde(default = "default_cfg")]
    cfg: f32,
    /// Random seed for reproducibility
    seed: Option<u64>,
    /// ComfyUI server URL
    #[serde(default = "default_server")]
    server: String,
    /// Python interpreter path (default: python3)
    python_path: Option<String>,
}

fn default_orientation() -> String {
    "landscape".to_string()
}
fn default_length() -> u32 {
    81
}
fn default_steps() -> u32 {
    20
}
fn default_cfg() -> f32 {
    5.0
}
fn default_server() -> String {
    "http://192.168.1.64:6000".to_string()
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode text_to_video"))?;

    let prompt = ctx.render(&p.prompt)?;
    let output_path = ctx.render(&p.output_path)?;
    let server = ctx.render(&p.server)?;
    let orientation = ctx.render(&p.orientation)?;

    // Resolution presets (latent dimensions - output is 2x)
    let (width, height) = match orientation.as_str() {
        "portrait" => (480, 832), // Output: 960x1664
        "square" => (640, 640),   // Output: 1280x1280
        _ => (832, 480),          // Output: 1664x960 (landscape)
    };

    let seed = p.seed.unwrap_or_else(rand::random);
    let python = p
        .python_path
        .as_ref()
        .map(|pp| ctx.render(pp))
        .transpose()?
        .unwrap_or_else(|| "python3".to_string());

    let script = video_gen_script(
        &server,
        &prompt,
        &output_path,
        width,
        height,
        p.length,
        p.steps,
        p.cfg,
        seed,
    );

    let status = Command::new(&python)
        .args(["-c", &script])
        .status()
        .with_context(|| ctx.error_context("spawn text_to_video python"))?;

    if !status.success() {
        anyhow::bail!(
            "Video generation failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn video_gen_script(
    server: &str,
    prompt: &str,
    output: &str,
    width: u32,
    height: u32,
    length: u32,
    steps: u32,
    cfg: f32,
    seed: u64,
) -> String {
    let prompt_escaped = prompt.replace('\\', "\\\\").replace('"', "\\\"");
    let negative = "blurry, low quality, distorted, watermark, text, deformed";

    format!(
        r#"
import requests
import time
import sys
from pathlib import Path

SERVER = "{server}"
PROMPT = "{prompt_escaped}"
NEGATIVE = "{negative}"
OUTPUT = "{output}"
WIDTH = {width}
HEIGHT = {height}
LENGTH = {length}
STEPS = {steps}
CFG = {cfg}
SEED = {seed}

workflow = {{
    "prompt": {{
        "1": {{"class_type": "UNETLoader",
              "inputs": {{"unet_name": "wan2.2_ti2v_5B_fp16.safetensors",
                         "weight_dtype": "fp8_e4m3fn"}}}},
        "2": {{"class_type": "CLIPLoader",
              "inputs": {{"clip_name": "umt5_xxl_fp8_e4m3fn_scaled.safetensors",
                         "type": "wan"}}}},
        "3": {{"class_type": "VAELoader",
              "inputs": {{"vae_name": "wan2.2_vae.safetensors"}}}},
        "4": {{"class_type": "CLIPTextEncode",
              "inputs": {{"text": PROMPT, "clip": ["2", 0]}}}},
        "5": {{"class_type": "CLIPTextEncode",
              "inputs": {{"text": NEGATIVE, "clip": ["2", 0]}}}},
        "6": {{"class_type": "WanImageToVideo",
              "inputs": {{
                  "positive": ["4", 0], "negative": ["5", 0],
                  "vae": ["3", 0], "width": WIDTH, "height": HEIGHT,
                  "length": LENGTH, "batch_size": 1}}}},
        "7": {{"class_type": "KSampler",
              "inputs": {{
                  "model": ["1", 0], "positive": ["6", 0],
                  "negative": ["6", 1], "latent_image": ["6", 2],
                  "seed": SEED, "steps": STEPS, "cfg": CFG,
                  "sampler_name": "euler", "scheduler": "normal",
                  "denoise": 1.0}}}},
        "8": {{"class_type": "VAEDecode",
              "inputs": {{"samples": ["7", 0], "vae": ["3", 0]}}}},
        "9": {{"class_type": "VHS_VideoCombine",
              "inputs": {{
                  "images": ["8", 0], "frame_rate": 16,
                  "loop_count": 0, "filename_prefix": "wan22",
                  "format": "video/h264-mp4", "pingpong": False,
                  "save_output": True}}}},
    }}
}}

print(f"Submitting to {{SERVER}}...")
print(f"  Prompt: {{PROMPT[:60]}}{{'...' if len(PROMPT) > 60 else ''}}")
print(f"  Latent: {{WIDTH}}x{{HEIGHT}} -> Output: {{WIDTH*2}}x{{HEIGHT*2}}")
print(f"  Frames: {{LENGTH}} ({{LENGTH/16:.1f}}s @ 16fps)")

r = requests.post(f"{{SERVER}}/prompt", json=workflow)
r.raise_for_status()
response = r.json()

if response.get("node_errors"):
    print(f"ERROR: {{response['node_errors']}}")
    sys.exit(1)

prompt_id = response["prompt_id"]
print(f"  Job ID: {{prompt_id}}")

print("  Generating (this takes ~13 minutes for 81 frames)", end="", flush=True)
while True:
    r = requests.get(f"{{SERVER}}/history/{{prompt_id}}")
    data = r.json()
    if prompt_id in data and data[prompt_id].get("status", {{}}).get("completed"):
        break
    print(".", end="", flush=True)
    time.sleep(30)
print(" done")

# Extract filename
outputs = data[prompt_id]["outputs"]
filename = None
for node_out in outputs.values():
    if "gifs" in node_out:
        filename = node_out["gifs"][0]["filename"]
        break

if filename is None:
    raise RuntimeError(f"No video output found in: {{outputs}}")

print(f"  Downloading: {{filename}}")
r = requests.get(f"{{SERVER}}/view", params={{"filename": filename, "subfolder": "", "type": "output"}})
r.raise_for_status()

Path(OUTPUT).parent.mkdir(parents=True, exist_ok=True)
with open(OUTPUT, "wb") as f:
    f.write(r.content)
print(f"Saved: {{OUTPUT}}")
"#
    )
}
