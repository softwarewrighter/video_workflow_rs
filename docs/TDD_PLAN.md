# TDD Plan (Red/Green)

The goal is an engine where “workflow YAML + inputs” → “artifacts + manifest”.

## Recommended loop

- Write a failing test in `vwf-core` for the smallest behavior.
- Implement minimal code to pass.
- Refactor only when green.
- Repeat.

## Suggested test sequence

1. **Parse** minimal workflow YAML into structs.
2. **Validate**: unknown step type → error includes step id.
3. **Render templates**: `{{var}}` substitution works; missing var errors.
4. **Write artifact** step: creates file; manifest references it.
5. **Dry run**: no filesystem mutation except manifest preview.
6. **Run**: full run writes manifest + artifacts.
7. **Command allowlist**: disallow unknown commands; show remediation.
8. **Capture logs**: stdout/stderr stored in `artifacts/`.
9. **LLM mock**: LLM step writes response file; validates JSON schema when requested.

## Non-functional requirements to keep

- Deterministic output paths (no random names unless explicitly requested).
- Full provenance: inputs, versions, step configs, environment snapshots.
- “Explainable failure”: errors should say *what* and *how to fix*.
