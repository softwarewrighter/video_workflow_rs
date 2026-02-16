#!/usr/bin/env python3
"""
FLUX text-to-image client for ComfyUI server.

Usage:
    python flux_client.py --prompt "Your prompt" --output image.png
    python flux_client.py --prompt "..." --orientation portrait --output short_bg.png
    python flux_client.py --prompt "..." --orientation landscape --output explainer_bg.png
"""

import argparse
import requests
import time
import random
import sys

SERVER = "http://192.168.1.64:8570"

# Resolution presets - exact aspect ratios, divisible by 8 for FLUX latent space
# 720x1280 is standard HD, exact 9:16/16:9, scales cleanly to 1080x1920/1920x1080
RESOLUTIONS = {
    "portrait": (720, 1280),   # 9:16 for YouTube Shorts (scales 1.5x to 1080x1920)
    "landscape": (1280, 720),  # 16:9 for explainers (scales 1.5x to 1920x1080)
    "square": (1024, 1024),
}


def generate_image(prompt: str, width: int, height: int, seed: int = None, prefix: str = "flux"):
    """Generate an image and return the PNG bytes and filename."""
    if seed is None:
        seed = random.randint(0, 2**32)

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
                  "inputs": {"images": ["6", 0], "filename_prefix": prefix}},
        }
    }

    print(f"Submitting to {SERVER}...")
    print(f"  Prompt: {prompt[:60]}{'...' if len(prompt) > 60 else ''}")
    print(f"  Size: {width}x{height}")
    print(f"  Seed: {seed}")

    # Submit
    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]
    print(f"  Job ID: {prompt_id}")

    # Poll for completion
    print("  Waiting for generation...", end="", flush=True)
    start = time.time()
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(2)

    elapsed = time.time() - start
    print(f" done ({elapsed:.1f}s)")

    # Extract filename
    outputs = data[prompt_id]["outputs"]
    for node_out in outputs.values():
        if "images" in node_out:
            img = node_out["images"][0]
            filename = img["filename"]
            break

    # Download
    print(f"  Downloading: {filename}")
    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output"
    })
    r.raise_for_status()
    return r.content, filename


def main():
    global SERVER

    parser = argparse.ArgumentParser(description="Generate images with FLUX")
    parser.add_argument("--prompt", "-p", required=True, help="Text prompt")
    parser.add_argument("--output", "-o", required=True, help="Output file path")
    parser.add_argument("--orientation", "-r", choices=["portrait", "landscape", "square"],
                       default="portrait", help="Image orientation (default: portrait)")
    parser.add_argument("--width", type=int, help="Custom width (overrides orientation)")
    parser.add_argument("--height", type=int, help="Custom height (overrides orientation)")
    parser.add_argument("--seed", "-s", type=int, help="Random seed for reproducibility")
    parser.add_argument("--server", default=SERVER, help=f"Server URL (default: {SERVER})")

    args = parser.parse_args()
    SERVER = args.server

    # Determine resolution
    if args.width and args.height:
        width, height = args.width, args.height
    else:
        width, height = RESOLUTIONS[args.orientation]

    # Generate
    png_bytes, _ = generate_image(args.prompt, width, height, args.seed)

    # Save
    with open(args.output, "wb") as f:
        f.write(png_bytes)
    print(f"Saved: {args.output}")


if __name__ == "__main__":
    main()
