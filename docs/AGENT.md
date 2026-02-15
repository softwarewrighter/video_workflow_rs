# Agent guide: implementing VWF

This repo is intentionally a **skeleton**. The first milestone is making the CLI run a small YAML workflow
deterministically and produce a `run.json` manifest in the chosen `--workdir`.

## Golden rules

1. **Do not expand scope while tests are red.**
2. Treat YAML as untrusted input: validate and fail with actionable messages.
3. No "magic defaults." If a parameter matters, it must be explicit.
4. No network calls in unit tests. Use mocks.

## Milestones

### M1: Workflow runner (no shell, no LLM)
- Parse workflow YAML
- Resolve templates against context vars
- Write artifacts to workdir
- Produce run manifest with step timings + status

### M2: Shell step
- Add `run_command` step with allowlist + explicit working directory
- Capture stdout/stderr to artifacts
- Provide a safe `--dry-run`

### M3: LLM adapter layer
- Define a trait `LlmClient` with a request struct (system/user/tools/attachments)
- Implement a `MockLlmClient` for tests
- Provide an optional "Claude Code CLI" adapter behind a feature flag:
  - binary path configured
  - arguments pinned
  - tool docs mounted as files
  - output captured as a file + parsed (when configured)

### M4: Web UI
- Use Yew to:
  - list workflows from `examples/workflows`
  - collect variables (mad-lib)
  - upload inputs
  - generate a `run request` JSON for the CLI (or call a local HTTP runner)

Keep the UI "dumb": it should *not* implement workflow logic; it should just produce config + inputs.
