# vwf-web

Yew/WASM UI skeleton.

## Run locally

Requires `trunk`:

```bash
cargo install trunk
trunk serve --open
```

Today it only exports a JSON "run request" for the CLI.
Later milestones can add:
- file uploads (stills, OBS clips)
- workflow picker
- a local HTTP runner that invokes `vwf-core` (or the CLI)
