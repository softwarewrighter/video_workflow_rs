#!/usr/bin/env python3
"""
Wan 2.2 text-to-video client for ComfyUI server.

Usage:
    python wan22_client.py --prompt "A cat playing" --output video.mp4
    python wan22_client.py --prompt "..." --output video.mp4 --orientation portrait
"""

import argparse
import requests
import time
import random
import sys

SERVER = "http://192.168.1.64:6000"

# Resolution presets (latent dimensions - output is 2x due to VAE upscale)
# Width/height must be multiples of 16
RESOLUTIONS = {
    "portrait": (480, 832),   # Output: 960x1664 (~9:16)
    "landscape": (832, 480),  # Output: 1664x960 (~16:9)
    "square": (640, 640),     # Output: 1280x1280
}


def generate_video(
    prompt: str,
    negative: str = "blurry, low quality, distorted, watermark, text, deformed",
    width: int = 832,
    height: int = 480,
    length: int = 81,
    steps: int = 20,
    cfg: float = 5.0,
    seed: int | None = None,
    prefix: str = "wan22",
) -> tuple[bytes, str]:
    """Generate a video from a text prompt. Returns (mp4_bytes, filename)."""
    if seed is None:
        seed = random.randint(0, 2**32)

    print(f"Submitting to {SERVER}...")
    print(f"  Prompt: {prompt[:60]}{'...' if len(prompt) > 60 else ''}")
    print(f"  Latent: {width}x{height} → Output: {width*2}x{height*2}")
    print(f"  Frames: {length} ({length/16:.1f}s @ 16fps)")
    print(f"  Seed: {seed}")

    workflow = {
        "prompt": {
            "1": {"class_type": "UNETLoader",
                  "inputs": {"unet_name": "wan2.2_ti2v_5B_fp16.safetensors",
                             "weight_dtype": "fp8_e4m3fn"}},
            "2": {"class_type": "CLIPLoader",
                  "inputs": {"clip_name": "umt5_xxl_fp8_e4m3fn_scaled.safetensors",
                             "type": "wan"}},
            "3": {"class_type": "VAELoader",
                  "inputs": {"vae_name": "wan2.2_vae.safetensors"}},
            "4": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": prompt, "clip": ["2", 0]}},
            "5": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": negative, "clip": ["2", 0]}},
            "6": {"class_type": "WanImageToVideo",
                  "inputs": {
                      "positive": ["4", 0], "negative": ["5", 0],
                      "vae": ["3", 0], "width": width, "height": height,
                      "length": length, "batch_size": 1,
                  }},
            "7": {"class_type": "KSampler",
                  "inputs": {
                      "model": ["1", 0], "positive": ["6", 0],
                      "negative": ["6", 1], "latent_image": ["6", 2],
                      "seed": seed, "steps": steps, "cfg": cfg,
                      "sampler_name": "euler", "scheduler": "normal",
                      "denoise": 1.0,
                  }},
            "8": {"class_type": "VAEDecode",
                  "inputs": {"samples": ["7", 0], "vae": ["3", 0]}},
            "9": {"class_type": "VHS_VideoCombine",
                  "inputs": {
                      "images": ["8", 0], "frame_rate": 16,
                      "loop_count": 0, "filename_prefix": prefix,
                      "format": "video/h264-mp4", "pingpong": False,
                      "save_output": True,
                  }},
        }
    }

    # Submit
    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    response = r.json()

    if response.get("node_errors"):
        print(f"ERROR: {response['node_errors']}")
        sys.exit(1)

    prompt_id = response["prompt_id"]
    print(f"  Job ID: {prompt_id}")

    # Poll (Wan 2.2 takes ~13 min for 81 frames)
    print("  Generating (this takes ~13 minutes for 81 frames)", end="", flush=True)
    start = time.time()
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(30)

    elapsed = time.time() - start
    print(f" done ({elapsed/60:.1f} min)")

    # Extract filename (video outputs are under 'gifs' key)
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

    parser = argparse.ArgumentParser(description="Generate video from text with Wan 2.2")
    parser.add_argument("--prompt", "-p", required=True, help="Text prompt")
    parser.add_argument("--output", "-o", required=True, help="Output video path")
    parser.add_argument("--orientation", "-r", choices=["portrait", "landscape", "square"],
                       default="landscape", help="Video orientation (default: landscape)")
    parser.add_argument("--width", type=int, help="Custom latent width (output is 2x)")
    parser.add_argument("--height", type=int, help="Custom latent height (output is 2x)")
    parser.add_argument("--length", "-n", type=int, default=81,
                       help="Frame count (default: 81 = ~5s at 16fps)")
    parser.add_argument("--steps", type=int, default=20, help="Sampling steps (default: 20)")
    parser.add_argument("--cfg", type=float, default=5.0, help="CFG scale (default: 5.0)")
    parser.add_argument("--seed", "-s", type=int, help="Random seed")
    parser.add_argument("--server", default=SERVER, help=f"Server URL (default: {SERVER})")

    args = parser.parse_args()
    SERVER = args.server

    # Determine resolution
    if args.width and args.height:
        width, height = args.width, args.height
    else:
        width, height = RESOLUTIONS[args.orientation]

    # Generate
    mp4_bytes, _ = generate_video(
        args.prompt,
        width=width,
        height=height,
        length=args.length,
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
