#!/usr/bin/env python3
"""
Text-to-video pipeline: FLUX (text→image) + SVD (image→video)

Requires both services to be available (run sequentially, not simultaneously).
"""

import argparse
import requests
import time
import random
import sys
from pathlib import Path

FLUX_SERVER = "http://192.168.1.64:8570"
SVD_SERVER = "http://192.168.1.64:8100"


def check_server(url: str, name: str) -> bool:
    """Check if a server is responding."""
    try:
        r = requests.get(f"{url}/system_stats", timeout=5)
        return r.status_code == 200
    except:
        return False


def generate_image(prompt: str, width: int, height: int, seed: int) -> bytes:
    """Generate image with FLUX."""
    workflow = {
        "prompt": {
            "1": {"class_type": "CheckpointLoaderSimple",
                  "inputs": {"ckpt_name": "flux1-schnell-fp8.safetensors"}},
            "2": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": prompt, "clip": ["1", 1]}},
            "3": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": "", "clip": ["1", 1]}},
            "4": {"class_type": "EmptyLatentImage",
                  "inputs": {"width": width, "height": height, "batch_size": 1}},
            "5": {"class_type": "KSampler",
                  "inputs": {"model": ["1", 0], "positive": ["2", 0],
                             "negative": ["3", 0], "latent_image": ["4", 0],
                             "seed": seed, "steps": 4, "cfg": 1.0,
                             "sampler_name": "euler", "scheduler": "simple",
                             "denoise": 1.0}},
            "6": {"class_type": "VAEDecode",
                  "inputs": {"samples": ["5", 0], "vae": ["1", 2]}},
            "7": {"class_type": "SaveImage",
                  "inputs": {"images": ["6", 0], "filename_prefix": "t2v"}},
        }
    }

    r = requests.post(f"{FLUX_SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]

    # Poll
    while True:
        r = requests.get(f"{FLUX_SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(2)

    # Get filename and download
    outputs = data[prompt_id]["outputs"]
    for node_out in outputs.values():
        if "images" in node_out:
            filename = node_out["images"][0]["filename"]
            break

    r = requests.get(f"{FLUX_SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output"
    })
    r.raise_for_status()
    return r.content


def generate_video(image_bytes: bytes, width: int, height: int,
                   frames: int, motion: int, fps: int, seed: int) -> bytes:
    """Generate video with SVD from image bytes."""

    # Upload image
    r = requests.post(
        f"{SVD_SERVER}/upload/image",
        files={"image": ("input.png", image_bytes, "image/png")},
        data={"subfolder": "", "type": "input"},
    )
    r.raise_for_status()
    image_name = r.json()["name"]

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
                      "video_frames": frames,
                      "motion_bucket_id": motion,
                      "fps": fps, "augmentation_level": 0.0,
                  }},
            "4": {"class_type": "KSampler",
                  "inputs": {
                      "model": ["1", 0], "positive": ["3", 0],
                      "negative": ["3", 1], "latent_image": ["3", 2],
                      "seed": seed, "steps": 20, "cfg": 2.5,
                      "sampler_name": "euler", "scheduler": "karras",
                      "denoise": 1.0,
                  }},
            "5": {"class_type": "VAEDecode",
                  "inputs": {"samples": ["4", 0], "vae": ["1", 2]}},
            "6": {"class_type": "VHS_VideoCombine",
                  "inputs": {
                      "images": ["5", 0], "frame_rate": fps,
                      "loop_count": 0, "filename_prefix": "t2v",
                      "format": "video/h264-mp4", "pingpong": False,
                      "save_output": True, "pix_fmt": "yuv420p",
                      "crf": 19, "save_metadata": True,
                      "trim_to_audio": False,
                  }},
        }
    }

    r = requests.post(f"{SVD_SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]

    # Poll
    while True:
        r = requests.get(f"{SVD_SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(5)

    # Get filename and download
    outputs = data[prompt_id]["outputs"]
    for node_out in outputs.values():
        if "gifs" in node_out:
            filename = node_out["gifs"][0]["filename"]
            break

    r = requests.get(f"{SVD_SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output"
    })
    r.raise_for_status()
    return r.content


def main():
    parser = argparse.ArgumentParser(description="Text to video via FLUX + SVD")
    parser.add_argument("--prompt", "-p", required=True, help="Text prompt")
    parser.add_argument("--output", "-o", required=True, help="Output video path")
    parser.add_argument("--orientation", "-r", choices=["portrait", "landscape", "square"],
                       default="portrait", help="Video orientation")
    parser.add_argument("--duration", "-d", type=float, default=5.0, help="Duration in seconds")
    parser.add_argument("--fps", type=int, default=6, help="Frame rate")
    parser.add_argument("--motion", "-m", type=int, default=80, help="Motion amount 1-300")
    parser.add_argument("--seed", "-s", type=int, help="Random seed")

    args = parser.parse_args()
    seed = args.seed or random.randint(0, 2**32)
    frames = int(args.duration * args.fps)

    # Resolution based on orientation
    if args.orientation == "portrait":
        width, height = 576, 1024  # 9:16
    elif args.orientation == "landscape":
        width, height = 1024, 576  # 16:9
    else:
        width, height = 768, 768

    print(f"=== Text-to-Video Pipeline ===")
    print(f"Prompt: {args.prompt[:60]}...")
    print(f"Size: {width}x{height} ({args.orientation})")
    print(f"Duration: {args.duration}s ({frames} frames @ {args.fps}fps)")
    print(f"Motion: {args.motion}")
    print(f"Seed: {seed}")
    print()

    # Check which server is up
    flux_up = check_server(FLUX_SERVER, "FLUX")
    svd_up = check_server(SVD_SERVER, "SVD")

    print(f"FLUX (text-to-image): {'✓ online' if flux_up else '✗ offline'}")
    print(f"SVD (image-to-video): {'✓ online' if svd_up else '✗ offline'}")
    print()

    if not flux_up and not svd_up:
        print("ERROR: Neither server is running!")
        sys.exit(1)

    if flux_up and svd_up:
        print("Both servers online - running full pipeline...")
    elif flux_up:
        print("Only FLUX online - will generate image only")
    else:
        print("Only SVD online - need FLUX for text-to-image step")
        print("Start FLUX server and try again, or provide an image with svd_client.py")
        sys.exit(1)

    # Step 1: Generate image
    print("\n[1/2] Generating image with FLUX", end="", flush=True)
    start = time.time()
    image_bytes = generate_image(args.prompt, width, height, seed)
    print(f" done ({time.time()-start:.1f}s)")

    # Save intermediate image
    img_path = Path(args.output).with_suffix(".png")
    with open(img_path, "wb") as f:
        f.write(image_bytes)
    print(f"  Saved: {img_path}")

    if not svd_up:
        print("\nSVD offline - image saved, video skipped")
        return

    # Step 2: Generate video
    print("\n[2/2] Generating video with SVD", end="", flush=True)
    start = time.time()
    video_bytes = generate_video(image_bytes, width, height, frames, args.motion, args.fps, seed)
    print(f" done ({time.time()-start:.1f}s)")

    with open(args.output, "wb") as f:
        f.write(video_bytes)
    print(f"  Saved: {args.output}")


if __name__ == "__main__":
    main()
