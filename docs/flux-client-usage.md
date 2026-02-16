# ComfyUI + FLUX.1 schnell — Remote Client Usage Guide

Connect to the text-to-image server from any host on the LAN via the REST API.

## Server Details

| Field | Value |
|---|---|
| Host | `192.168.1.64` |
| Port | `8570` |
| Base URL | `http://192.168.1.64:8570` |
| Model | `flux1-schnell-fp8.safetensors` (FLUX.1 schnell, fp8 quantized) |
| License | Apache 2.0 — commercial use of generated images is unrestricted |
| GPU | NVIDIA RTX 5060 Ti 16 GB |
| Generation time | ~12–15 s per image (varies with resolution) |

## Quick Test

```bash
curl http://192.168.1.64:8570/system_stats
```

A JSON response with `comfyui_version` confirms the server is reachable.

---

## API Reference

### 1. Submit a Prompt — `POST /prompt`

Queue an image generation job. Returns immediately with a `prompt_id`.

```bash
curl -s -X POST http://192.168.1.64:8570/prompt \
  -H "Content-Type: application/json" \
  -d @prompt.json
```

**Response:**

```json
{
  "prompt_id": "d4984eeb-1dbe-4631-a1ca-a4f3edc44ba5",
  "number": 0,
  "node_errors": {}
}
```

- `prompt_id` — use this to poll for completion
- `number` — queue position
- `node_errors` — non-empty if the workflow is invalid

### 2. Poll for Completion — `GET /history/{prompt_id}`

```bash
curl -s http://192.168.1.64:8570/history/{prompt_id}
```

**Response when complete:**

```json
{
  "{prompt_id}": {
    "outputs": {
      "7": {
        "images": [
          {
            "filename": "my_image_00001_.png",
            "subfolder": "",
            "type": "output"
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

An empty JSON object `{}` means the job is still running. Poll every 2–3 seconds.

### 3. Download the Image — `GET /view`

```bash
curl -s "http://192.168.1.64:8570/view?filename={filename}&subfolder=&type=output" \
  -o output.png
```

### 4. Check Queue Status — `GET /queue`

```bash
curl -s http://192.168.1.64:8570/queue
```

```json
{
  "queue_running": [],
  "queue_pending": []
}
```

### 5. System Stats — `GET /system_stats`

Returns GPU VRAM usage, ComfyUI version, PyTorch version, etc.

---

## Workflow JSON (Prompt Format)

ComfyUI uses a node-graph workflow serialized as JSON. Each key is a node ID
(arbitrary string), and edges are expressed as `["source_node_id", output_index]`.

### Minimal FLUX.1 schnell Workflow

```json
{
  "prompt": {
    "1": {
      "class_type": "CheckpointLoaderSimple",
      "inputs": {
        "ckpt_name": "flux1-schnell-fp8.safetensors"
      }
    },
    "2": {
      "class_type": "CLIPTextEncode",
      "inputs": {
        "text": "YOUR PROMPT HERE",
        "clip": ["1", 1]
      }
    },
    "3": {
      "class_type": "CLIPTextEncode",
      "inputs": {
        "text": "",
        "clip": ["1", 1]
      }
    },
    "4": {
      "class_type": "EmptyLatentImage",
      "inputs": {
        "width": 768,
        "height": 1344,
        "batch_size": 1
      }
    },
    "5": {
      "class_type": "KSampler",
      "inputs": {
        "model": ["1", 0],
        "positive": ["2", 0],
        "negative": ["3", 0],
        "latent_image": ["4", 0],
        "seed": 42,
        "steps": 4,
        "cfg": 1.0,
        "sampler_name": "euler",
        "scheduler": "simple",
        "denoise": 1.0
      }
    },
    "6": {
      "class_type": "VAEDecode",
      "inputs": {
        "samples": ["5", 0],
        "vae": ["1", 2]
      }
    },
    "7": {
      "class_type": "SaveImage",
      "inputs": {
        "images": ["6", 0],
        "filename_prefix": "output"
      }
    }
  }
}
```

### Node Graph (data flow)

```
CheckpointLoaderSimple ──┬── MODEL ──────────── KSampler ── VAEDecode ── SaveImage
  (node 1)               ├── CLIP ── CLIPTextEncode (positive, node 2)
                          │           CLIPTextEncode (negative, node 3)
                          └── VAE ─────────────────────────── VAEDecode
                                                EmptyLatentImage (node 4)
```

---

## Tunable Parameters

### Resolution (node 4 — EmptyLatentImage)

| Use Case | width | height | Notes |
|---|---|---|---|
| YouTube Short (portrait) | 768 | 1344 | 9:16 aspect ratio |
| Square | 1024 | 1024 | Good default |
| Landscape 16:9 | 1344 | 768 | |
| Small / fast test | 512 | 512 | ~2x faster |

Higher resolutions use more VRAM. Stay at or below 1344 on the long edge for 16 GB.

### KSampler (node 5)

| Parameter | Recommended | Notes |
|---|---|---|
| `steps` | **4** | FLUX.1 schnell is designed for 1–4 steps. More steps do not improve quality. |
| `cfg` | **1.0** | FLUX schnell ignores classifier-free guidance. Always use 1.0. |
| `sampler_name` | `euler` | Best for schnell's few-step generation. |
| `scheduler` | `simple` | Use `simple` for schnell. |
| `seed` | any integer | Same seed + same prompt = same image. Use random for variety. |
| `denoise` | `1.0` | Always 1.0 for txt2img. |

### filename_prefix (node 7 — SaveImage)

The server appends `_00001_.png`, `_00002_.png`, etc. Use a unique prefix per
request to avoid filename collisions when downloading.

---

## Full Agent Example (bash)

End-to-end script: submit prompt, poll for completion, download result.

```bash
#!/usr/bin/env bash
set -euo pipefail

SERVER="http://192.168.1.64:8570"
PROMPT_TEXT="A grand university library with towering bookshelves and warm lamplight, photorealistic, portrait orientation"
SEED=$RANDOM
PREFIX="agent_$(date +%s)"

# 1. Submit
RESPONSE=$(curl -s -X POST "$SERVER/prompt" \
  -H "Content-Type: application/json" \
  -d "$(cat <<EOF
{
  "prompt": {
    "1": {"class_type": "CheckpointLoaderSimple", "inputs": {"ckpt_name": "flux1-schnell-fp8.safetensors"}},
    "2": {"class_type": "CLIPTextEncode", "inputs": {"text": "$PROMPT_TEXT", "clip": ["1", 1]}},
    "3": {"class_type": "CLIPTextEncode", "inputs": {"text": "", "clip": ["1", 1]}},
    "4": {"class_type": "EmptyLatentImage", "inputs": {"width": 768, "height": 1344, "batch_size": 1}},
    "5": {"class_type": "KSampler", "inputs": {"model": ["1", 0], "positive": ["2", 0], "negative": ["3", 0], "latent_image": ["4", 0], "seed": $SEED, "steps": 4, "cfg": 1.0, "sampler_name": "euler", "scheduler": "simple", "denoise": 1.0}},
    "6": {"class_type": "VAEDecode", "inputs": {"samples": ["5", 0], "vae": ["1", 2]}},
    "7": {"class_type": "SaveImage", "inputs": {"images": ["6", 0], "filename_prefix": "$PREFIX"}}
  }
}
EOF
)")

PROMPT_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['prompt_id'])")
echo "Submitted prompt_id: $PROMPT_ID"

# 2. Poll for completion
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

  if [ "$COMPLETED" = "yes" ]; then
    break
  fi
  echo "Waiting..."
  sleep 3
done

# 3. Extract filename and download
FILENAME=$(echo "$HISTORY" | python3 -c "
import sys, json
d = json.load(sys.stdin)['$PROMPT_ID']
for node_id, out in d['outputs'].items():
    if 'images' in out:
        print(out['images'][0]['filename'])
        break
")

echo "Downloading: $FILENAME"
curl -s "$SERVER/view?filename=$FILENAME&subfolder=&type=output" -o "$FILENAME"
echo "Saved to: $FILENAME"
```

## Python Agent Example

```python
import requests
import time
import random

SERVER = "http://192.168.1.64:8570"

def generate_image(prompt: str, width=768, height=1344, seed=None, prefix="agent"):
    """Generate an image and return the PNG bytes."""
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

    # Submit
    r = requests.post(f"{SERVER}/prompt", json=workflow)
    r.raise_for_status()
    prompt_id = r.json()["prompt_id"]

    # Poll
    while True:
        r = requests.get(f"{SERVER}/history/{prompt_id}")
        data = r.json()
        if prompt_id in data and data[prompt_id]["status"].get("completed"):
            break
        time.sleep(2)

    # Extract filename
    outputs = data[prompt_id]["outputs"]
    for node_out in outputs.values():
        if "images" in node_out:
            img = node_out["images"][0]
            filename = img["filename"]
            break

    # Download
    r = requests.get(f"{SERVER}/view", params={
        "filename": filename, "subfolder": "", "type": "output"
    })
    r.raise_for_status()
    return r.content, filename


# Usage
if __name__ == "__main__":
    png_bytes, fname = generate_image(
        "A science laboratory with colorful beakers and microscopes, photorealistic"
    )
    with open(fname, "wb") as f:
        f.write(png_bytes)
    print(f"Saved: {fname}")
```

---

## Server Management (run from the server host)

```bash
# Start
cd ~/tools/tti/stable-diffusion-webui-docker
docker compose --profile comfy up -d

# Stop
docker compose --profile comfy down

# Logs (live)
docker logs -f webui-docker-comfy-1

# Rebuild (after Dockerfile changes)
docker compose --profile comfy build --no-cache
docker compose --profile comfy up -d
```

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Connection refused | Container not running | `docker compose --profile comfy up -d` |
| `node_errors` in submit response | Invalid workflow JSON | Check node IDs and edge references |
| CUDA OOM | Resolution too high | Reduce width/height; stay ≤ 1344 on long edge |
| Slow first generation | Model loading into VRAM | Normal (~10 s overhead on first run); subsequent generations are faster |
| Empty `/history/{id}` response | Job still running | Keep polling every 2–3 s |
