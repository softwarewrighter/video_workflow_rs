#!/usr/bin/env python3
"""
SVD (Stable Video Diffusion) image-to-video client for ComfyUI server.

Usage:
    python svd_client.py --input image.jpg --output video.mp4
    python svd_client.py --input image.jpg --output video.mp4 --motion 150 --frames 30
"""

import argparse
import requests
import time
import random
from pathlib import Path

SERVER = "http://192.168.1.64:8100"


def generate_video(
    image_path: str,
    width: int = 576,
    height: int = 1024,
    video_frames: int = 14,
    motion_bucket_id: int = 127,
    fps: int = 6,
    augmentation_level: float = 0.0,
    steps: int = 20,
    cfg: float = 2.5,
    seed: int | None = None,
    prefix: str = "svd",
) -> tuple[bytes, str]:
    """Generate a video from an image. Returns (mp4_bytes, filename)."""
    if seed is None:
        seed = random.randint(0, 2**32)

    print(f"Uploading: {image_path}")

    # Upload image
    with open(image_path, "rb") as f:
        r = requests.post(
            f"{SERVER}/upload/image",
            files={"image": (Path(image_path).name, f)},
            data={"subfolder": "", "type": "input"},
        )
    r.raise_for_status()
    image_name = r.json()["name"]
    print(f"  Uploaded as: {image_name}")

    # Build workflow
    workflow = {
        "prompt": {
            "1": {"class_type": "ImageOnlyCheckpointLoader",
                  "inputs": {"ckpt_name": "svd_xt.safetensors"}},
            "2": {"class_type": "LoadImage",
                  "inputs": {"image": image_name}},
            "3": {"class_type": "SVD_img2vid_Conditioning",
                  "inputs": {
                      "clip_vision": ["1", 1], "init_image": ["2", 0],
                      "vae": ["1", 2], "width": width, "height": height,
                      "video_frames": video_frames,
                      "motion_bucket_id": motion_bucket_id,
                      "fps": fps, "augmentation_level": augmentation_level,
                  }},
            "4": {"class_type": "KSampler",
                  "inputs": {
                      "model": ["1", 0], "positive": ["3", 0],
                      "negative": ["3", 1], "latent_image": ["3", 2],
                      "seed": seed, "steps": steps, "cfg": cfg,
                      "sampler_name": "euler", "scheduler": "karras",
                      "denoise": 1.0,
                  }},
            "5": {"class_type": "VAEDecode",
                  "inputs": {"samples": ["4", 0], "vae": ["1", 2]}},
            "6": {"class_type": "VHS_VideoCombine",
                  "inputs": {
                      "images": ["5", 0], "frame_rate": fps,
                      "loop_count": 0, "filename_prefix": prefix,
                      "format": "video/h264-mp4", "pingpong": False,
                      "save_output": True, "pix_fmt": "yuv420p",
                      "crf": 19, "save_metadata": True,
                      "trim_to_audio": False,
                  }},
        }
    }

    # Submit
    print(f"Submitting to {SERVER}...")
    print(f"  Size: {width}x{height}")
    print(f"  Frames: {video_frames} ({video_frames/fps:.1f}s @ {fps}fps)")
    print(f"  Motion: {motion_bucket_id}")
    print(f"  Seed: {seed}")

    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]
    print(f"  Job ID: {prompt_id}")

    # Poll (SVD takes ~70s for 14 frames, longer for more)
    print("  Generating", end="", flush=True)
    start = time.time()
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(5)

    elapsed = time.time() - start
    print(f" done ({elapsed:.0f}s)")

    # Extract filename (SVD uses 'gifs' key, not 'images')
    outputs = data[prompt_id]["outputs"]
    filename = None
    for node_out in outputs.values():
        if "gifs" in node_out:
            filename = node_out["gifs"][0]["filename"]
            break

    if filename is None:
        raise RuntimeError(f"No video output found in: {outputs}")

    # Download
    print(f"  Downloading: {filename}")
    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output",
    })
    r.raise_for_status()
    return r.content, filename


def main():
    global SERVER

    parser = argparse.ArgumentParser(description="Generate video from image with SVD")
    parser.add_argument("--input", "-i", required=True, help="Input image path")
    parser.add_argument("--output", "-o", required=True, help="Output video path")
    parser.add_argument("--width", type=int, help="Output width (default: match input)")
    parser.add_argument("--height", type=int, help="Output height (default: match input)")
    parser.add_argument("--frames", "-n", type=int, default=14, help="Number of frames (default: 14)")
    parser.add_argument("--fps", type=int, default=6, help="Frame rate (default: 6)")
    parser.add_argument("--motion", "-m", type=int, default=127,
                       help="Motion bucket ID 1-1023 (default: 127, lower=subtle, higher=dramatic)")
    parser.add_argument("--augment", "-a", type=float, default=0.0,
                       help="Augmentation level 0-10 (default: 0, higher=more creative)")
    parser.add_argument("--steps", type=int, default=20, help="Sampling steps (default: 20)")
    parser.add_argument("--cfg", type=float, default=2.5, help="CFG scale (default: 2.5)")
    parser.add_argument("--seed", "-s", type=int, help="Random seed")
    parser.add_argument("--server", default=SERVER, help=f"Server URL (default: {SERVER})")

    args = parser.parse_args()
    SERVER = args.server

    # Get image dimensions if not specified
    from PIL import Image
    img = Image.open(args.input)
    width = args.width or img.width
    height = args.height or img.height

    # Ensure dimensions are reasonable for 16GB VRAM
    max_dim = 1024
    if width > max_dim or height > max_dim:
        scale = max_dim / max(width, height)
        width = int(width * scale)
        height = int(height * scale)
        # Round to nearest 8 for latent space
        width = (width // 8) * 8
        height = (height // 8) * 8
        print(f"Scaled to {width}x{height} for VRAM limits")

    # Generate
    mp4_bytes, _ = generate_video(
        args.input,
        width=width,
        height=height,
        video_frames=args.frames,
        motion_bucket_id=args.motion,
        fps=args.fps,
        augmentation_level=args.augment,
        steps=args.steps,
        cfg=args.cfg,
        seed=args.seed,
    )

    # Save
    with open(args.output, "wb") as f:
        f.write(mp4_bytes)
    print(f"Saved: {args.output}")


if __name__ == "__main__":
    main()
