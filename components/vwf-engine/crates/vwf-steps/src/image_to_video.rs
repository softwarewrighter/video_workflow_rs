//! Handler for image_to_video step kind.
//!
//! Generates video from a still image using SVD-XT via ComfyUI REST API.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Input image path
    input_path: String,
    /// Output video path (.mp4)
    output_path: String,
    /// Number of frames to generate (default: 14)
    #[serde(default = "default_frames")]
    frames: u32,
    /// Frame rate (default: 6)
    #[serde(default = "default_fps")]
    fps: u32,
    /// Motion bucket ID 1-1023 (default: 127, lower=subtle, higher=dramatic)
    #[serde(default = "default_motion")]
    motion: u32,
    /// Random seed for reproducibility
    seed: Option<u64>,
    /// ComfyUI server URL
    #[serde(default = "default_server")]
    server: String,
    /// Python interpreter path (default: python3)
    python_path: Option<String>,
}

fn default_frames() -> u32 { 14 }
fn default_fps() -> u32 { 6 }
fn default_motion() -> u32 { 127 }
fn default_server() -> String { "http://192.168.1.64:8100".to_string() }

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode image_to_video"))?;

    let input_path = ctx.render(&p.input_path)?;
    let output_path = ctx.render(&p.output_path)?;
    let server = ctx.render(&p.server)?;

    let seed = p.seed.unwrap_or_else(|| rand::random());
    let python = p.python_path
        .as_ref()
        .map(|pp| ctx.render(pp))
        .transpose()?
        .unwrap_or_else(|| "python3".to_string());

    let script = video_gen_script(
        &server, &input_path, &output_path,
        p.frames, p.fps, p.motion, seed,
    );

    let status = Command::new(&python)
        .args(["-c", &script])
        .status()
        .with_context(|| ctx.error_context("spawn image_to_video python"))?;

    if !status.success() {
        anyhow::bail!("Video generation failed with exit code: {:?}", status.code());
    }

    Ok(())
}

fn video_gen_script(
    server: &str, input: &str, output: &str,
    frames: u32, fps: u32, motion: u32, seed: u64,
) -> String {
    format!(r#"
import requests
import time
from pathlib import Path
from PIL import Image

SERVER = "{server}"
INPUT = "{input}"
OUTPUT = "{output}"
FRAMES = {frames}
FPS = {fps}
MOTION = {motion}
SEED = {seed}

# Get image dimensions and scale for VRAM
img = Image.open(INPUT)
width, height = img.width, img.height
max_dim = 1024
if width > max_dim or height > max_dim:
    scale = max_dim / max(width, height)
    width = int(width * scale)
    height = int(height * scale)
width = (width // 8) * 8
height = (height // 8) * 8

# Upload image
print(f"Uploading: {{INPUT}}")
with open(INPUT, "rb") as f:
    r = requests.post(f"{{SERVER}}/upload/image",
        files={{"image": (Path(INPUT).name, f)}},
        data={{"subfolder": "", "type": "input"}})
r.raise_for_status()
image_name = r.json()["name"]
print(f"  Uploaded as: {{image_name}}")

workflow = {{
    "prompt": {{
        "1": {{"class_type": "ImageOnlyCheckpointLoader",
              "inputs": {{"ckpt_name": "svd_xt.safetensors"}}}},
        "2": {{"class_type": "LoadImage",
              "inputs": {{"image": image_name}}}},
        "3": {{"class_type": "SVD_img2vid_Conditioning",
              "inputs": {{
                  "clip_vision": ["1", 1], "init_image": ["2", 0],
                  "vae": ["1", 2], "width": width, "height": height,
                  "video_frames": FRAMES, "motion_bucket_id": MOTION,
                  "fps": FPS, "augmentation_level": 0.0}}}},
        "4": {{"class_type": "KSampler",
              "inputs": {{
                  "model": ["1", 0], "positive": ["3", 0],
                  "negative": ["3", 1], "latent_image": ["3", 2],
                  "seed": SEED, "steps": 20, "cfg": 2.5,
                  "sampler_name": "euler", "scheduler": "karras",
                  "denoise": 1.0}}}},
        "5": {{"class_type": "VAEDecode",
              "inputs": {{"samples": ["4", 0], "vae": ["1", 2]}}}},
        "6": {{"class_type": "VHS_VideoCombine",
              "inputs": {{
                  "images": ["5", 0], "frame_rate": FPS,
                  "loop_count": 0, "filename_prefix": "svd",
                  "format": "video/h264-mp4", "pingpong": False,
                  "save_output": True, "pix_fmt": "yuv420p",
                  "crf": 19, "save_metadata": True, "trim_to_audio": False}}}},
    }}
}}

print(f"Submitting to {{SERVER}}...")
print(f"  Size: {{width}}x{{height}}, Frames: {{FRAMES}} ({{FRAMES/FPS:.1f}}s @ {{FPS}}fps)")
r = requests.post(f"{{SERVER}}/prompt", json=workflow)
r.raise_for_status()
prompt_id = r.json()["prompt_id"]
print(f"  Job ID: {{prompt_id}}")

print("  Generating", end="", flush=True)
while True:
    r = requests.get(f"{{SERVER}}/history/{{prompt_id}}")
    data = r.json()
    if prompt_id in data and data[prompt_id].get("status", {{}}).get("completed"):
        break
    print(".", end="", flush=True)
    time.sleep(5)
print(" done")

# Extract filename (SVD uses 'gifs' key)
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
"#)
}
