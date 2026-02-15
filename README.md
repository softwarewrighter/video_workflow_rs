# video-workflow-rs (VWF)

A Rust framework for *repeatable* video-production workflows with strict, minimal-context AI calls.

## Goals

- **Inversion of control:** workflows are data (YAML/JSON), the runner is code.
- **Determinism & auditability:** every run produces a manifest + logs + artifacts.
- **AI calls as “steps”:** prompts are templated and stored; responses are captured and validated.
- **TDD-friendly:** core logic is unit-tested with mocks (no network, no shell by default).
- **Extensible:** add new step kinds without rewriting the runner.

## What’s in this repo

- `crates/vwf-core` — workflow engine + config schema + step library.
- `crates/vwf-cli`  — command-line runner (`vwf`).
- `crates/vwf-web`  — Yew/WASM UI skeleton (uploads + workflow selection).

## Quick start

```bash
cargo test
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo --dry-run
cargo run -p vwf-cli -- run examples/workflows/shorts_narration.yaml --workdir work/demo
```

The first run uses `--dry-run` to show what would happen; the second run will create artifacts.

## Philosophy

Your agents are unreliable because *the world is huge* and “helpful guessing” is the default failure mode.
This project forces:
- small prompts
- pinned parameters
- explicit inputs/outputs per step
- validation gates
- provenance logs

## Next steps for your coding agent

See `docs/AGENT.md` and `docs/TDD_PLAN.md`.
