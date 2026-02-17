#!/usr/bin/env python3
"""
FLUX generative zoom - creates keyframes with AI-generated detail.

Takes an image and generates zoom keyframes where the AI fills in detail
as you zoom in, rather than just scaling/blurring existing pixels.
"""

import argparse
import requests
import time
import random
import base64
from pathlib import Path
from PIL import Image
import io

SERVER = "http://192.168.1.64:8570"


def upload_image(image_path: str) -> str:
    """Upload image to ComfyUI and return the filename."""
    with open(image_path, "rb") as f:
        files = {"image": (Path(image_path).name, f, "image/jpeg")}
        r = requests.post(f"{SERVER}/upload/image", files=files)
        r.raise_for_status()
        return r.json()["name"]


def generate_img2img(input_filename: str, prompt: str, width: int, height: int,
                     seed: int, denoise: float = 0.4) -> bytes:
    """Generate image using img2img with FLUX."""

    workflow = {
        "prompt": {
            # Load checkpoint
            "1": {"class_type": "CheckpointLoaderSimple",
                  "inputs": {"ckpt_name": "flux1-schnell-fp8.safetensors"}},
            # Load input image
            "10": {"class_type": "LoadImage",
                   "inputs": {"image": input_filename}},
            # Resize image to target dimensions
            "11": {"class_type": "ImageScale",
                   "inputs": {"image": ["10", 0], "width": width, "height": height,
                             "upscale_method": "lanczos", "crop": "center"}},
            # Encode image to latent
            "12": {"class_type": "VAEEncode",
                   "inputs": {"pixels": ["11", 0], "vae": ["1", 2]}},
            # Text prompts
            "2": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": prompt, "clip": ["1", 1]}},
            "3": {"class_type": "CLIPTextEncode",
                  "inputs": {"text": "", "clip": ["1", 1]}},
            # KSampler with image input (img2img)
            "5": {"class_type": "KSampler",
                  "inputs": {"model": ["1", 0], "positive": ["2", 0],
                             "negative": ["3", 0], "latent_image": ["12", 0],
                             "seed": seed, "steps": 4, "cfg": 1.0,
                             "sampler_name": "euler", "scheduler": "simple",
                             "denoise": denoise}},
            # Decode and save
            "6": {"class_type": "VAEDecode",
                  "inputs": {"samples": ["5", 0], "vae": ["1", 2]}},
            "7": {"class_type": "SaveImage",
                  "inputs": {"images": ["6", 0], "filename_prefix": "zoom"}},
        }
    }

    # Submit
    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]

    # Poll for completion
    print(".", end="", flush=True)
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id].get("status", {}).get("completed"):
            break
        print(".", end="", flush=True)
        time.sleep(1)

    # Extract filename and download
    outputs = data[prompt_id]["outputs"]
    for node_out in outputs.values():
        if "images" in node_out:
            filename = node_out["images"][0]["filename"]
            break

    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output"
    })
    r.raise_for_status()
    return r.content


def crop_center(img: Image.Image, crop_ratio: float) -> Image.Image:
    """Crop to center by given ratio (0.5 = crop to 50% of original)."""
    w, h = img.size
    new_w = int(w * crop_ratio)
    new_h = int(h * crop_ratio)
    left = (w - new_w) // 2
    top = (h - new_h) // 2
    return img.crop((left, top, left + new_w, top + new_h))


def main():
    global SERVER

    parser = argparse.ArgumentParser(description="Generate zoom keyframes with AI detail")
    parser.add_argument("--input", "-i", required=True, help="Input image path")
    parser.add_argument("--output-dir", "-o", required=True, help="Output directory for keyframes")
    parser.add_argument("--prompt", "-p", default="", help="Text prompt to guide generation")
    parser.add_argument("--frames", "-n", type=int, default=5, help="Number of keyframes (default: 5)")
    parser.add_argument("--zoom", "-z", type=float, default=2.0, help="Total zoom factor (default: 2.0)")
    parser.add_argument("--denoise", "-d", type=float, default=0.35, help="Denoise strength 0-1 (default: 0.35)")
    parser.add_argument("--seed", "-s", type=int, help="Random seed for reproducibility")
    parser.add_argument("--server", default=SERVER, help=f"Server URL (default: {SERVER})")

    args = parser.parse_args()
    SERVER = args.server
    seed = args.seed or random.randint(0, 2**32)

    # Load input image
    input_img = Image.open(args.input)
    width, height = input_img.size
    print(f"Input: {args.input} ({width}x{height})")
    print(f"Generating {args.frames} keyframes with {args.zoom}x zoom")
    print(f"Denoise: {args.denoise}, Seed: {seed}")

    # Create output directory
    out_dir = Path(args.output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    # Calculate crop ratios for each frame
    # Frame 0: full image (ratio=1.0)
    # Frame N: zoomed in (ratio=1/zoom)
    crop_ratios = [1.0 - (1.0 - 1.0/args.zoom) * i / (args.frames - 1)
                   for i in range(args.frames)]

    # Generate keyframes
    current_img = input_img

    for i, ratio in enumerate(crop_ratios):
        print(f"\nFrame {i+1}/{args.frames} (crop ratio: {ratio:.2f})", end="", flush=True)

        if i == 0:
            # First frame is just the original
            out_path = out_dir / f"frame_{i:03d}.png"
            current_img.save(out_path)
            print(f" saved (original)")
        else:
            # Crop current image
            cropped = crop_center(current_img, ratio / crop_ratios[i-1])

            # Save cropped image temporarily for upload
            temp_path = out_dir / f"_temp_crop.png"
            cropped.save(temp_path)

            # Upload and run img2img
            filename = upload_image(str(temp_path))

            # Generate with AI
            png_bytes = generate_img2img(
                filename, args.prompt, width, height, seed + i, args.denoise
            )

            # Save result
            out_path = out_dir / f"frame_{i:03d}.png"
            with open(out_path, "wb") as f:
                f.write(png_bytes)

            # Load for next iteration
            current_img = Image.open(io.BytesIO(png_bytes))

            print(f" saved")

            # Clean up temp
            temp_path.unlink()

    print(f"\nDone! Keyframes saved to {out_dir}/")
    print(f"\nTo create video from keyframes:")
    print(f"  ffmpeg -framerate 1 -i {out_dir}/frame_%03d.png -vf 'minterpolate=fps=25:mi_mode=mci' -c:v libx264 -pix_fmt yuv420p zoom.mp4")


if __name__ == "__main__":
    main()
