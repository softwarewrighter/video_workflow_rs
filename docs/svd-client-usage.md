# ComfyUI + SVD-XT — Remote Client Usage Guide (Image-to-Video)

Connect to the image-to-video server from any host on the LAN via the REST API.

## Server Details

| Field | Value |
|---|---|
| Host | `192.168.1.64` |
| Port | `8100` |
| Base URL | `http://192.168.1.64:8100` |
| Model | `svd_xt.safetensors` (Stable Video Diffusion XT, 25-frame capable) |
| GPU | NVIDIA RTX 5060 Ti 16 GB |
| Generation time | ~70 s for 14 frames, ~90 s for 18 frames (576x1024) |
| Output | H.264 MP4 (also supports WebM, GIF, ProRes, HEVC) |

> **Important:** This is a separate Docker container from the FLUX text-to-image
> service. Only one GPU service should run at a time. Stop FLUX before starting SVD
> and vice versa.

## Quick Test

```bash
curl http://192.168.1.64:8100/system_stats
```

## Architecture Note — Separate from FLUX

The SVD and FLUX services are **independent Docker containers** (Option B design):

| Service | Directory | Port | Container |
|---|---|---|---|
| FLUX (text-to-image) | `~/tools/tti/stable-diffusion-webui-docker` | 8570 | `webui-docker-comfy-1` |
| SVD (image-to-video) | `~/tools/tti/ittv` | 8100 | `ittv-comfy-svd` |

Run one at a time to avoid GPU contention.

---

## API Reference

The API endpoints are identical to the FLUX service (both are ComfyUI). The
differences are in the **workflow JSON** and the **output format** (video instead
of images).

### 1. Upload Input Image — `POST /upload/image`

SVD requires an input image. Upload it first.

```bash
curl -s -X POST http://192.168.1.64:8100/upload/image \
  -F "image=@my_photo.jpg" \
  -F "subfolder=" \
  -F "type=input"
```

**Response:**

```json
{
  "name": "my_photo.jpg",
  "subfolder": "",
  "type": "input"
}
```

Use the returned `name` in the `LoadImage` node.

Alternatively, place files directly in the container's input directory on the
server host:

```bash
# From server host:
docker exec ittv-comfy-svd chmod 777 /data/config/comfy/input/
cp my_photo.jpg ~/tools/tti/ittv/data/config/comfy/input/
```

### 2. Submit a Prompt — `POST /prompt`

```bash
curl -s -X POST http://192.168.1.64:8100/prompt \
  -H "Content-Type: application/json" \
  -d @svd_prompt.json
```

**Response:**

```json
{
  "prompt_id": "78e309ae-286e-41ae-a273-d44fd2b20920",
  "number": 0,
  "node_errors": {}
}
```

### 3. Poll for Completion — `GET /history/{prompt_id}`

```bash
curl -s http://192.168.1.64:8100/history/{prompt_id}
```

**Response when complete:**

```json
{
  "{prompt_id}": {
    "outputs": {
      "6": {
        "gifs": [
          {
            "filename": "my_video_00001.mp4",
            "subfolder": "",
            "type": "output",
            "format": "video/h264-mp4",
            "frame_rate": 6.0
          }
        ]
      }
    },
    "status": {
      "status_str": "success",
      "completed": true
    }
  }
}
```

> **Note:** SVD video outputs appear under the `gifs` key (not `images`). This is
> a VideoHelperSuite naming convention — the content is still an MP4 file.

An empty JSON object `{}` means the job is still running. Poll every 5 seconds
(SVD jobs take ~70 s).

### 4. Download the Video — `GET /view`

```bash
curl -s "http://192.168.1.64:8100/view?filename={filename}&subfolder=&type=output" \
  -o output.mp4
```

### 5. Check Queue / System Stats

```bash
curl -s http://192.168.1.64:8100/queue
curl -s http://192.168.1.64:8100/system_stats
```

---

## Workflow JSON — SVD Image-to-Video

### Minimal SVD-XT Workflow

```json
{
  "prompt": {
    "1": {
      "class_type": "ImageOnlyCheckpointLoader",
      "inputs": {
        "ckpt_name": "svd_xt.safetensors"
      }
    },
    "2": {
      "class_type": "LoadImage",
      "inputs": {
        "image": "INPUT_FILENAME.jpg"
      }
    },
    "3": {
      "class_type": "SVD_img2vid_Conditioning",
      "inputs": {
        "clip_vision": ["1", 1],
        "init_image": ["2", 0],
        "vae": ["1", 2],
        "width": 576,
        "height": 1024,
        "video_frames": 14,
        "motion_bucket_id": 127,
        "fps": 6,
        "augmentation_level": 0.0
      }
    },
    "4": {
      "class_type": "KSampler",
      "inputs": {
        "model": ["1", 0],
        "positive": ["3", 0],
        "negative": ["3", 1],
        "latent_image": ["3", 2],
        "seed": 42,
        "steps": 20,
        "cfg": 2.5,
        "sampler_name": "euler",
        "scheduler": "karras",
        "denoise": 1.0
      }
    },
    "5": {
      "class_type": "VAEDecode",
      "inputs": {
        "samples": ["4", 0],
        "vae": ["1", 2]
      }
    },
    "6": {
      "class_type": "VHS_VideoCombine",
      "inputs": {
        "images": ["5", 0],
        "frame_rate": 6,
        "loop_count": 0,
        "filename_prefix": "svd_output",
        "format": "video/h264-mp4",
        "pingpong": false,
        "save_output": true,
        "pix_fmt": "yuv420p",
        "crf": 19,
        "save_metadata": true,
        "trim_to_audio": false
      }
    }
  }
}
```

### Node Graph (data flow)

```
ImageOnlyCheckpointLoader ──┬── MODEL ──────────────── KSampler ── VAEDecode ─┐
  (node 1)                  ├── CLIP_VISION ──┐                                │
                            └── VAE ──────────┤                                │
                                              ▼                                │
LoadImage ────────────── SVD_img2vid_Conditioning                              │
  (node 2)                    (node 3)                                         │
                         ├── positive ──► KSampler (node 4)                    │
                         ├── negative ──► KSampler                             │
                         └── latent ────► KSampler                             │
                                                                               │
                                                          VHS_VideoCombine ◄───┘
                                                            (node 6)
                                                            ▼ MP4 file
```

---

## Tunable Parameters

### SVD_img2vid_Conditioning (node 3)

| Parameter | Default | Range | Notes |
|---|---|---|---|
| `width` | 576 | 16–16384 | Output video width. Match input aspect ratio. |
| `height` | 1024 | 16–16384 | Output video height. Match input aspect ratio. |
| `video_frames` | 14 | 1–4096 | Number of frames. 14 = ~2.3 s @ 6fps. 18 = ~3 s. |
| `motion_bucket_id` | 127 | 1–1023 | Amount of motion. Lower = subtler/smoother. Higher = more dramatic. |
| `fps` | 6 | 1–1024 | Affects temporal conditioning (how the model perceives time). |
| `augmentation_level` | 0.0 | 0.0–10.0 | Noise added to input image. 0 = faithful. Higher = more creative divergence. |

### Resolution Presets (16 GB VRAM safe)

| Input Aspect | width | height | Duration (14 frames) | Duration (18 frames) |
|---|---|---|---|---|
| Portrait 9:16 | 576 | 1024 | 2.3 s | 3.0 s |
| Landscape 16:9 | 1024 | 576 | 2.3 s | 3.0 s |
| Square | 768 | 768 | 2.3 s | 3.0 s |
| Small / fast test | 512 | 512 | 2.3 s | 3.0 s |

> Stay at or below 1024 on the long edge for 16 GB VRAM. Going larger risks OOM.

### KSampler (node 4)

| Parameter | Recommended | Notes |
|---|---|---|
| `steps` | **20** | Good balance of quality vs speed. 15 is faster, 25 slightly better. |
| `cfg` | **2.5** | Guidance scale. 2.0–3.0 is the sweet spot for SVD. Higher = sharper but can artifact. |
| `sampler_name` | `euler` | Standard choice for SVD. |
| `scheduler` | `karras` | Recommended for SVD (unlike FLUX which uses `simple`). |
| `seed` | any integer | Same seed + same input = same video. Randomize for variety. |
| `denoise` | `1.0` | Always 1.0 for img2vid. |

### VHS_VideoCombine (node 6)

| Parameter | Default | Notes |
|---|---|---|
| `frame_rate` | 6 | Playback FPS. 6 is cinematic/slow. 8–12 for smoother playback. |
| `format` | `video/h264-mp4` | Also: `video/h265-mp4`, `video/av1-webm`, `image/gif`, `video/ProRes` |
| `pingpong` | false | If true, plays forward then backward (doubles apparent length). |
| `crf` | 19 | Quality (0 = lossless, 51 = worst). 19 is high quality. |
| `filename_prefix` | — | Unique per request to avoid collisions. |

### Motion Control Tips

| Desired Effect | `motion_bucket_id` | `augmentation_level` | Notes |
|---|---|---|---|
| Subtle zoom / parallax | 60–80 | 0.0–0.02 | Gentle, smooth camera-like motion |
| Moderate animation | 100–150 | 0.0 | Default behavior, noticeable motion |
| Dramatic motion | 200–300 | 0.0–0.05 | More scene dynamics, risk of artifacts |
| Near-static (Ken Burns) | 30–50 | 0.0 | Very subtle drift |

### Generation Time Estimates (576x1024, 20 steps)

| Frames | Duration @ 6fps | Approx. Generation Time |
|---|---|---|
| 14 | 2.3 s | ~70 s |
| 18 | 3.0 s | ~90 s |
| 25 | 4.2 s | ~125 s |

---

## Full Agent Example (bash)

End-to-end: upload image, submit SVD prompt, poll, download MP4.

```bash
#!/usr/bin/env bash
set -euo pipefail

SERVER="http://192.168.1.64:8100"
INPUT_IMAGE="$1"           # path to local image file
SEED=${2:-$RANDOM}
PREFIX="svd_$(date +%s)"

# 1. Upload input image
UPLOAD_RESP=$(curl -s -X POST "$SERVER/upload/image" \
  -F "image=@$INPUT_IMAGE" -F "subfolder=" -F "type=input")
IMAGE_NAME=$(echo "$UPLOAD_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['name'])")
echo "Uploaded: $IMAGE_NAME"

# 2. Submit SVD workflow
RESPONSE=$(curl -s -X POST "$SERVER/prompt" \
  -H "Content-Type: application/json" \
  -d "$(cat <<EOF
{
  "prompt": {
    "1": {"class_type": "ImageOnlyCheckpointLoader", "inputs": {"ckpt_name": "svd_xt.safetensors"}},
    "2": {"class_type": "LoadImage", "inputs": {"image": "$IMAGE_NAME"}},
    "3": {"class_type": "SVD_img2vid_Conditioning", "inputs": {
      "clip_vision": ["1", 1], "init_image": ["2", 0], "vae": ["1", 2],
      "width": 576, "height": 1024, "video_frames": 14,
      "motion_bucket_id": 80, "fps": 6, "augmentation_level": 0.0
    }},
    "4": {"class_type": "KSampler", "inputs": {
      "model": ["1", 0], "positive": ["3", 0], "negative": ["3", 1],
      "latent_image": ["3", 2], "seed": $SEED, "steps": 20, "cfg": 2.5,
      "sampler_name": "euler", "scheduler": "karras", "denoise": 1.0
    }},
    "5": {"class_type": "VAEDecode", "inputs": {"samples": ["4", 0], "vae": ["1", 2]}},
    "6": {"class_type": "VHS_VideoCombine", "inputs": {
      "images": ["5", 0], "frame_rate": 6, "loop_count": 0,
      "filename_prefix": "$PREFIX", "format": "video/h264-mp4",
      "pingpong": false, "save_output": true, "pix_fmt": "yuv420p",
      "crf": 19, "save_metadata": true, "trim_to_audio": false
    }}
  }
}
EOF
)")

PROMPT_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['prompt_id'])")
echo "Submitted prompt_id: $PROMPT_ID"

# 3. Poll for completion (SVD takes ~70s)
while true; do
  HISTORY=$(curl -s "$SERVER/history/$PROMPT_ID")
  COMPLETED=$(echo "$HISTORY" | python3 -c "
import sys, json
d = json.load(sys.stdin)
if '$PROMPT_ID' in d and d['$PROMPT_ID'].get('status', {}).get('completed'):
    print('yes')
else:
    print('no')
" 2>/dev/null || echo "no")

  if [ "$COMPLETED" = "yes" ]; then break; fi
  echo "Generating video..."
  sleep 5
done

# 4. Extract filename (SVD outputs are under 'gifs' key, not 'images')
FILENAME=$(echo "$HISTORY" | python3 -c "
import sys, json
d = json.load(sys.stdin)['$PROMPT_ID']
for node_id, out in d['outputs'].items():
    if 'gifs' in out:
        print(out['gifs'][0]['filename'])
        break
")

# 5. Download
echo "Downloading: $FILENAME"
curl -s "$SERVER/view?filename=$FILENAME&subfolder=&type=output" -o "$FILENAME"
echo "Saved to: $FILENAME"
```

**Usage:**

```bash
chmod +x svd_generate.sh
./svd_generate.sh my_photo.jpg
./svd_generate.sh my_photo.jpg 12345    # with explicit seed
```

## Python Agent Example

```python
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
    motion_bucket_id: int = 80,
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

    # Upload image
    with open(image_path, "rb") as f:
        r = requests.post(
            f"{SERVER}/upload/image",
            files={"image": (Path(image_path).name, f)},
            data={"subfolder": "", "type": "input"},
        )
    r.raise_for_status()
    image_name = r.json()["name"]

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
    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]

    # Poll (SVD takes ~70s)
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id]["status"].get("completed"):
            break
        time.sleep(5)

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
    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output",
    })
    r.raise_for_status()
    return r.content, filename


# Usage
if __name__ == "__main__":
    mp4_bytes, fname = generate_video(
        "my_photo.jpg",
        motion_bucket_id=80,      # subtle zoom
        video_frames=14,           # 2.3s @ 6fps
    )
    with open(fname, "wb") as f:
        f.write(mp4_bytes)
    print(f"Saved: {fname}")
```

---

## Key Differences from the FLUX (text-to-image) API

| | FLUX (text-to-image) | SVD (image-to-video) |
|---|---|---|
| Port | 8570 | 8100 |
| Input | Text prompt | Image file (upload first) |
| Checkpoint loader | `CheckpointLoaderSimple` | `ImageOnlyCheckpointLoader` |
| Conditioning | `CLIPTextEncode` | `SVD_img2vid_Conditioning` |
| Output node | `SaveImage` | `VHS_VideoCombine` |
| Output key in history | `outputs.*.images[]` | `outputs.*.gifs[]` |
| Output format | PNG | MP4 (H.264) |
| KSampler steps | 4 | 20 |
| KSampler cfg | 1.0 | 2.5 |
| KSampler scheduler | `simple` | `karras` |
| Generation time | ~12 s | ~70 s (14 frames) |

---

## Server Management (run from the server host)

```bash
# Start SVD
cd ~/tools/tti/ittv
docker compose up -d

# Stop SVD
docker compose down

# Logs
docker logs -f ittv-comfy-svd

# Rebuild
docker compose build --no-cache
docker compose up -d
```

### One-service-at-a-time workflow

```bash
# Stop FLUX, start SVD
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy down
cd ~/tools/tti/ittv && docker compose up -d

# Stop SVD, start FLUX
cd ~/tools/tti/ittv && docker compose down
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy up -d
```

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Connection refused | Container not running | `docker compose up -d` from `~/tools/tti/ittv` |
| `node_errors` / missing `pingpong` | Incomplete VHS_VideoCombine inputs | Ensure `pingpong` and `save_output` fields are present |
| CUDA OOM | Resolution or frame count too high | Reduce width/height to 576x1024 max; reduce `video_frames` |
| `gifs` key missing from output | Wrong output node | Use `VHS_VideoCombine`, not `SaveImage` |
| Blurry / low quality video | Too few steps or low cfg | Use steps=20, cfg=2.5 minimum |
| Too much motion / artifacts | `motion_bucket_id` too high | Lower to 60–80 for subtle zoom effects |
| Image not found | Not uploaded | Upload via `/upload/image` or copy to `ittv/data/config/comfy/input/` |
| Port conflict with FLUX | Both services running | Stop one before starting the other |
| Empty `/history/{id}` | Job still running | SVD takes ~70 s; poll every 5 s |
