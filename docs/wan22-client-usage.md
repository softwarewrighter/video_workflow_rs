# ComfyUI + Wan 2.2 TI2V 5B — Remote Client Usage Guide (Text-to-Video)

Connect to the text-to-video server from any host on the LAN via the REST API.

## Server Details

| Field | Value |
|---|---|
| Host | `192.168.1.64` |
| Port | `6000` |
| Base URL | `http://192.168.1.64:6000` |
| Diffusion model | `wan2.2_ti2v_5B_fp16.safetensors` (loaded as fp8_e4m3fn) |
| Text encoder | `umt5_xxl_fp8_e4m3fn_scaled.safetensors` |
| VAE | `wan2.2_vae.safetensors` |
| License | Apache 2.0 — commercial use of generated videos is unrestricted |
| GPU | NVIDIA RTX 5060 Ti 16 GB |
| VRAM usage | ~11.5 GB during generation |
| Generation time | ~13 min for 81 frames (5 s video), ~32 s per sampling step |
| Output | H.264 MP4 via VideoHelperSuite (also supports WebM, GIF, ProRes) |

> **Important:** This is a separate Docker container from the FLUX and SVD services.
> Only one GPU service should run at a time to avoid GPU contention.

## Quick Test

```bash
curl http://192.168.1.64:6000/system_stats
```

A JSON response with `comfyui_version` confirms the server is reachable.

---

## Architecture Note — Separate Containers

| Service | Directory | Port | Container |
|---|---|---|---|
| FLUX (text-to-image) | `~/tools/tti/stable-diffusion-webui-docker` | 8570 | `webui-docker-comfy-1` |
| SVD (image-to-video) | `~/tools/tti/ittv` | 8100 | `ittv-comfy-svd` |
| **Wan 2.2 (text-to-video)** | **`~/tools/tti/comfy-wan22`** | **6000** | **`comfy-wan22`** |

Run one at a time to avoid GPU contention.

---

## API Reference

The API endpoints are identical to the other ComfyUI services. The differences are
in the **workflow JSON** and the **model loading pattern** (Wan 2.2 uses three
separate loaders instead of a single checkpoint).

### 1. Submit a Prompt — `POST /prompt`

Queue a video generation job. Returns immediately with a `prompt_id`.

```bash
curl -s -X POST http://192.168.1.64:6000/prompt \
  -H "Content-Type: application/json" \
  -d @wan22_prompt.json
```

**Response:**

```json
{
  "prompt_id": "44a90b14-11e4-4766-a4a6-4ffa9ca4a78c",
  "number": 0,
  "node_errors": {}
}
```

- `prompt_id` — use this to poll for completion
- `number` — queue position
- `node_errors` — non-empty if the workflow is invalid

### 2. Poll for Completion — `GET /history/{prompt_id}`

```bash
curl -s http://192.168.1.64:6000/history/{prompt_id}
```

**Response when complete:**

```json
{
  "{prompt_id}": {
    "outputs": {
      "9": {
        "gifs": [
          {
            "filename": "wan22_eagle_00001.mp4",
            "subfolder": "",
            "type": "output",
            "format": "video/h264-mp4",
            "frame_rate": 16.0
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

> **Note:** Video outputs appear under the `gifs` key (not `images`). This is a
> VideoHelperSuite naming convention — the content is still an MP4 file.

An empty JSON object `{}` means the job is still running. Poll every **15–30 seconds**
(Wan 2.2 jobs take ~13 minutes).

### 3. Download the Video — `GET /view`

```bash
curl -s "http://192.168.1.64:6000/view?filename={filename}&subfolder=&type=output" \
  -o output.mp4
```

### 4. Check Queue / System Stats

```bash
curl -s http://192.168.1.64:6000/queue
curl -s http://192.168.1.64:6000/system_stats
```

---

## Workflow JSON — Text-to-Video

Wan 2.2 uses **three separate model loaders** (unlike FLUX/SVD which use a single
checkpoint loader):

- `UNETLoader` — diffusion model (loaded as fp8 to fit 16 GB VRAM)
- `CLIPLoader` — UMT5 text encoder (type must be set to `"wan"`)
- `VAELoader` — Wan 2.2 VAE (includes 2x spatial upscale)

### Minimal Wan 2.2 Text-to-Video Workflow

```json
{
  "prompt": {
    "1": {
      "class_type": "UNETLoader",
      "inputs": {
        "unet_name": "wan2.2_ti2v_5B_fp16.safetensors",
        "weight_dtype": "fp8_e4m3fn"
      }
    },
    "2": {
      "class_type": "CLIPLoader",
      "inputs": {
        "clip_name": "umt5_xxl_fp8_e4m3fn_scaled.safetensors",
        "type": "wan"
      }
    },
    "3": {
      "class_type": "VAELoader",
      "inputs": {
        "vae_name": "wan2.2_vae.safetensors"
      }
    },
    "4": {
      "class_type": "CLIPTextEncode",
      "inputs": {
        "text": "YOUR PROMPT HERE",
        "clip": ["2", 0]
      }
    },
    "5": {
      "class_type": "CLIPTextEncode",
      "inputs": {
        "text": "blurry, low quality, distorted, watermark, text",
        "clip": ["2", 0]
      }
    },
    "6": {
      "class_type": "WanImageToVideo",
      "inputs": {
        "positive": ["4", 0],
        "negative": ["5", 0],
        "vae": ["3", 0],
        "width": 832,
        "height": 480,
        "length": 81,
        "batch_size": 1
      }
    },
    "7": {
      "class_type": "KSampler",
      "inputs": {
        "model": ["1", 0],
        "positive": ["6", 0],
        "negative": ["6", 1],
        "latent_image": ["6", 2],
        "seed": 42,
        "steps": 20,
        "cfg": 5.0,
        "sampler_name": "euler",
        "scheduler": "normal",
        "denoise": 1.0
      }
    },
    "8": {
      "class_type": "VAEDecode",
      "inputs": {
        "samples": ["7", 0],
        "vae": ["3", 0]
      }
    },
    "9": {
      "class_type": "VHS_VideoCombine",
      "inputs": {
        "images": ["8", 0],
        "frame_rate": 16,
        "loop_count": 0,
        "filename_prefix": "wan22_output",
        "format": "video/h264-mp4",
        "pingpong": false,
        "save_output": true
      }
    }
  }
}
```

### Node Graph (data flow)

```
UNETLoader ──── MODEL ──────────────────────────── KSampler ─── VAEDecode ─┐
  (node 1)                                           (node 7)    (node 8)  │
                                                       ▲  ▲  ▲             │
CLIPLoader ──── CLIP ──┬── CLIPTextEncode (pos, 4)     │  │  │             │
  (node 2)             └── CLIPTextEncode (neg, 5)     │  │  │             │
                                    │  │               │  │  │             │
VAELoader ──── VAE ──┐              │  │               │  │  │             │
  (node 3)           │              ▼  ▼               │  │  │             │
                     └──► WanImageToVideo (node 6) ────┘  │  │             │
                               ├── positive ──────────────┘  │             │
                               ├── negative ─────────────────┘             │
                               └── latent ─────────────────(see above)     │
                                                                           │
                                                       VHS_VideoCombine ◄──┘
                                                         (node 9)
                                                         ▼ MP4 file
```

---

## Tunable Parameters

### Resolution — WanImageToVideo (node 6)

The Wan 2.2 VAE includes a **2x spatial upscale**, so the final video is double the
latent resolution specified here.

| Latent (width x height) | Output Resolution | Aspect | Notes |
|---|---|---|---|
| 832 x 480 | 1664 x 960 | ~16:9 landscape | Recommended default |
| 480 x 832 | 960 x 1664 | ~9:16 portrait | Vertical video |
| 640 x 640 | 1280 x 1280 | 1:1 square | |
| 480 x 480 | 960 x 960 | 1:1 square | Faster, less VRAM |

> Width and height must be multiples of 16.

### Frame Count — `length` (node 6)

| length | Duration @ 16fps | Approx. Generation Time | Notes |
|---|---|---|---|
| 21 | 1.3 s | ~3.5 min | Quick test |
| 41 | 2.6 s | ~6.5 min | Short clip |
| 61 | 3.8 s | ~10 min | Medium |
| **81** | **5.1 s** | **~13 min** | **Default — 5-second video** |
| 101 | 6.3 s | ~16 min | May approach VRAM limit |

> The `length` parameter increments in steps of 4 (e.g., 21, 25, 29, ... 81, 85).
> Stay at or below 81 frames on 16 GB VRAM at 832x480 resolution.

### KSampler (node 7)

| Parameter | Recommended | Notes |
|---|---|---|
| `steps` | **20** | Good balance. 15 is faster but lower quality. 25 is marginally better. |
| `cfg` | **5.0** | Classifier-free guidance. 4.0–7.0 works well. Higher = more prompt adherence but can artifact. |
| `sampler_name` | `euler` | Standard choice for Wan 2.2. |
| `scheduler` | `normal` | Use `normal` for Wan 2.2 (unlike SVD which uses `karras`). |
| `seed` | any integer | Same seed + same prompt = same video. Randomize for variety. |
| `denoise` | `1.0` | Always 1.0 for text-to-video. |

### UNETLoader weight_dtype (node 1)

| Weight dtype | VRAM for model | Quality | Notes |
|---|---|---|---|
| `fp8_e4m3fn` | ~4.8 GB | Good | **Recommended for 16 GB.** Enables longer videos. |
| `fp8_e4m3fn_fast` | ~4.8 GB | Good | Faster inference, slightly less accurate. |
| `fp8_e5m2` | ~4.8 GB | Slightly lower | Larger dynamic range but fewer mantissa bits. |
| `default` (fp16) | ~9.5 GB | Best | Full precision. Risk of OOM on longer videos. |

### VHS_VideoCombine (node 9)

| Parameter | Default | Notes |
|---|---|---|
| `frame_rate` | 16 | Playback FPS. 16 is Wan 2.2's native rate. |
| `format` | `video/h264-mp4` | Also: `video/h265-mp4`, `video/av1-webm`, `image/gif`, `video/ProRes` |
| `pingpong` | false | If true, plays forward then backward (doubles apparent length). |
| `filename_prefix` | — | Unique per request to avoid collisions. |

### Negative Prompt Tips

A good negative prompt improves output quality:

```
blurry, low quality, distorted, watermark, text, deformed, ugly, oversaturated
```

---

## Full Agent Example (bash)

End-to-end: submit prompt, poll for completion, download MP4.

```bash
#!/usr/bin/env bash
set -euo pipefail

SERVER="http://192.168.1.64:6000"
PROMPT_TEXT="A majestic golden eagle soaring through a vivid blue sky above snow-capped mountains, cinematic lighting, smooth camera tracking shot"
NEGATIVE="blurry, low quality, distorted, watermark, text"
SEED=$RANDOM
PREFIX="wan22_$(date +%s)"
WIDTH=832
HEIGHT=480
LENGTH=81   # ~5 seconds at 16fps

# 1. Submit
RESPONSE=$(curl -s -X POST "$SERVER/prompt" \
  -H "Content-Type: application/json" \
  -d "$(cat <<EOF
{
  "prompt": {
    "1": {"class_type": "UNETLoader", "inputs": {"unet_name": "wan2.2_ti2v_5B_fp16.safetensors", "weight_dtype": "fp8_e4m3fn"}},
    "2": {"class_type": "CLIPLoader", "inputs": {"clip_name": "umt5_xxl_fp8_e4m3fn_scaled.safetensors", "type": "wan"}},
    "3": {"class_type": "VAELoader", "inputs": {"vae_name": "wan2.2_vae.safetensors"}},
    "4": {"class_type": "CLIPTextEncode", "inputs": {"text": "$PROMPT_TEXT", "clip": ["2", 0]}},
    "5": {"class_type": "CLIPTextEncode", "inputs": {"text": "$NEGATIVE", "clip": ["2", 0]}},
    "6": {"class_type": "WanImageToVideo", "inputs": {
      "positive": ["4", 0], "negative": ["5", 0], "vae": ["3", 0],
      "width": $WIDTH, "height": $HEIGHT, "length": $LENGTH, "batch_size": 1
    }},
    "7": {"class_type": "KSampler", "inputs": {
      "model": ["1", 0], "positive": ["6", 0], "negative": ["6", 1],
      "latent_image": ["6", 2], "seed": $SEED, "steps": 20, "cfg": 5.0,
      "sampler_name": "euler", "scheduler": "normal", "denoise": 1.0
    }},
    "8": {"class_type": "VAEDecode", "inputs": {"samples": ["7", 0], "vae": ["3", 0]}},
    "9": {"class_type": "VHS_VideoCombine", "inputs": {
      "images": ["8", 0], "frame_rate": 16, "loop_count": 0,
      "filename_prefix": "$PREFIX", "format": "video/h264-mp4",
      "pingpong": false, "save_output": true
    }}
  }
}
EOF
)")

PROMPT_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['prompt_id'])")
echo "Submitted prompt_id: $PROMPT_ID"

# 2. Poll for completion (Wan 2.2 takes ~13 min for 81 frames)
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
  echo "Generating video... (this takes ~13 minutes)"
  sleep 30
done

# 3. Extract filename (video outputs are under 'gifs' key)
FILENAME=$(echo "$HISTORY" | python3 -c "
import sys, json
d = json.load(sys.stdin)['$PROMPT_ID']
for node_id, out in d['outputs'].items():
    if 'gifs' in out:
        print(out['gifs'][0]['filename'])
        break
")

# 4. Download
echo "Downloading: $FILENAME"
curl -s "$SERVER/view?filename=$FILENAME&subfolder=&type=output" -o "$FILENAME"
echo "Saved to: $FILENAME"
```

## Python Agent Example

```python
import requests
import time
import random

SERVER = "http://192.168.1.64:6000"


def generate_video(
    prompt: str,
    negative: str = "blurry, low quality, distorted, watermark, text",
    width: int = 832,
    height: int = 480,
    length: int = 81,
    steps: int = 20,
    cfg: float = 5.0,
    seed: int | None = None,
    prefix: str = "wan22",
    weight_dtype: str = "fp8_e4m3fn",
) -> tuple[bytes, str]:
    """Generate a video from a text prompt. Returns (mp4_bytes, filename)."""
    if seed is None:
        seed = random.randint(0, 2**32)

    workflow = {
        "prompt": {
            "1": {"class_type": "UNETLoader",
                  "inputs": {"unet_name": "wan2.2_ti2v_5B_fp16.safetensors",
                             "weight_dtype": weight_dtype}},
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
    prompt_id = r.json()["prompt_id"]

    # Poll (Wan 2.2 takes ~13 min for 81 frames)
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id]["status"].get("completed"):
            break
        time.sleep(30)

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
    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output",
    })
    r.raise_for_status()
    return r.content, filename


# Usage
if __name__ == "__main__":
    mp4_bytes, fname = generate_video(
        "A serene Japanese garden with cherry blossoms falling into a koi pond, "
        "soft sunlight filtering through branches, cinematic 4K",
        length=41,   # ~2.6 seconds (faster for testing)
    )
    with open(fname, "wb") as f:
        f.write(mp4_bytes)
    print(f"Saved: {fname}")
```

---

## Key Differences from FLUX and SVD

| | FLUX (text-to-image) | SVD (image-to-video) | Wan 2.2 (text-to-video) |
|---|---|---|---|
| Port | 8570 | 8100 | **6000** |
| Input | Text prompt | Image file | **Text prompt** |
| Model loading | `CheckpointLoaderSimple` | `ImageOnlyCheckpointLoader` | **`UNETLoader` + `CLIPLoader` + `VAELoader`** |
| Text encoding | `CLIPTextEncode` | N/A (image-conditioned) | **`CLIPTextEncode`** (type `wan`) |
| Conditioning | N/A | `SVD_img2vid_Conditioning` | **`WanImageToVideo`** |
| Output node | `SaveImage` | `VHS_VideoCombine` | **`VHS_VideoCombine`** |
| Output key | `outputs.*.images[]` | `outputs.*.gifs[]` | **`outputs.*.gifs[]`** |
| Output format | PNG | MP4 | **MP4** |
| Output resolution | Same as latent | Same as latent | **2x latent** (VAE upscale) |
| Native FPS | N/A | 6 | **16** |
| KSampler steps | 4 | 20 | **20** |
| KSampler cfg | 1.0 | 2.5 | **5.0** |
| KSampler scheduler | `simple` | `karras` | **`normal`** |
| Generation time | ~12 s | ~70 s (14 frames) | **~13 min (81 frames)** |

---

## Image-to-Video Mode (Optional)

The Wan 2.2 TI2V model also supports **image-to-video** by providing a start image.
This requires an additional CLIP Vision model (not installed by default).

To enable image-to-video:

1. Download a CLIP Vision model (e.g., `clip_vision_h.safetensors`) to
   `comfy-wan22/data/models/clip_vision/`
2. Add `clip_vision: models/clip_vision` to `extra_model_paths.yaml`
3. Add these nodes to the workflow:
   - `LoadImage` — load the input image
   - `CLIPVisionLoader` — load the vision encoder
   - `CLIPVisionEncode` — encode the image
4. Connect the `clip_vision_output` and `start_image` optional inputs on
   `WanImageToVideo` (node 6)

Text-to-video works without these extras.

---

## Server Management (run from the server host)

```bash
# Start Wan 2.2
cd ~/tools/tti/comfy-wan22
docker compose up -d

# Stop Wan 2.2
docker compose down

# Logs (live)
docker logs -f comfy-wan22

# Rebuild (after Dockerfile changes)
docker compose build --no-cache
docker compose up -d
```

### One-service-at-a-time workflow

```bash
# Stop FLUX, start Wan 2.2
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy down
cd ~/tools/tti/comfy-wan22 && docker compose up -d

# Stop SVD, start Wan 2.2
cd ~/tools/tti/ittv && docker compose down
cd ~/tools/tti/comfy-wan22 && docker compose up -d

# Stop Wan 2.2, start FLUX
cd ~/tools/tti/comfy-wan22 && docker compose down
cd ~/tools/tti/stable-diffusion-webui-docker && docker compose --profile comfy up -d
```

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Connection refused | Container not running | `docker compose up -d` from `~/tools/tti/comfy-wan22` |
| `node_errors` in submit response | Invalid workflow JSON | Check node IDs and edge references |
| CUDA OOM | Resolution or frame count too high | Reduce width/height; reduce `length`; use `fp8_e4m3fn` |
| Very slow (>15 min) | High frame count or resolution | Use length=41 (2.6 s) for faster tests |
| `gifs` key missing from output | Wrong output node | Use `VHS_VideoCombine`, not `SaveImage` |
| Model not found | Model files not in `data/models/` | Verify files in `diffusion_models/`, `clip/`, `vae/` |
| Port conflict | Another service on port 6000 | Stop other services; check with `docker ps` |
| Empty `/history/{id}` | Job still running | Wan 2.2 takes ~13 min; poll every 30 s |
| First run very slow | Models loading from disk to GPU | Normal; ~30 s overhead. Subsequent jobs reuse cached models. |
| CLIPLoader error | Missing `type` parameter | Must set `"type": "wan"` in CLIPLoader inputs |
