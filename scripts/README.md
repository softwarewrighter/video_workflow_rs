# Scripts

Utility scripts for media generation and workflow execution.

## Build & Test

| Script | Description |
|--------|-------------|
| `build-all.sh` | Build all components |
| `test-all.sh` | Run all tests |

## Demo Scripts

| Script | Description |
|--------|-------------|
| `demo-short.sh` | Generate sample YouTube Short |
| `demo-explainer.sh` | Generate sample explainer video |
| `demo-flux.sh` | Generate images in all orientations |

## GPU Service Clients

These Python scripts connect to remote ComfyUI services.

| Script | Service | Port | Use Case |
|--------|---------|------|----------|
| `flux_client.py` | FLUX | 8570 | Text → Image |
| `svd_client.py` | SVD | 8100 | Image → Video |
| `wan22_client.py` | Wan 2.2 | 6000 | Text → Video |
| `text_to_video.py` | FLUX + SVD | — | Text → Image → Video (chained) |
| `flux_zoom.py` | FLUX | 8570 | Generative zoom (experimental) |

### Usage Examples

```bash
# Text-to-image (FLUX)
python scripts/flux_client.py \
  --prompt "A futuristic cityscape at sunset" \
  --orientation landscape \
  --output cityscape.png

# Image-to-video (SVD)
python scripts/svd_client.py \
  --input photo.jpg \
  --output animated.mp4 \
  --motion 100 \
  --frames 30

# Text-to-video (Wan 2.2)
python scripts/wan22_client.py \
  --prompt "Ocean waves crashing on rocky shore" \
  --orientation landscape \
  --output waves.mp4

# Music generation (midi-cli-rs)
midi-cli-rs preset --mood upbeat --duration 5 -o intro.wav
```

## Dependencies

- Python 3.10+ with `requests` and `pillow`
- Remote ComfyUI server at 192.168.1.64
- FluidSynth for music generation (midi-cli-rs)

## GPU Service Notes

Only one GPU service can run at a time (shared 16GB VRAM). Check service status:

```bash
curl -s http://192.168.1.64:8570/system_stats  # FLUX
curl -s http://192.168.1.64:8100/system_stats  # SVD
curl -s http://192.168.1.64:6000/system_stats  # Wan 2.2
```
