# Product Requirements Document (PRD)

## Problem Statement

Video production workflows involving AI agents suffer from "agent drift":
- Agents forget steps
- Agents hallucinate or guess parameters
- Agents make similar mistakes repeatedly
- Documentation alone does not prevent these errors

The current process for producing YouTube Shorts (portrait, 1-3 min) and Explainer videos (landscape, 1-20 min) requires:
- Title images and text
- Background music (selective)
- OBS clips
- VHS CLI (tape) recordings
- Intro/outro videos
- "Like and subscribe" segments
- Still extraction for narration generation
- Git repo analysis for hooks, slides, summaries
- Mad-lib style templates with fill-in-the-blanks
- Review files (description.txt, narration.txt, slide-outline.txt)

## Solution

A Rust-based framework with **inversion of control** that:
1. Invokes AI agents with limited context and focused prompts
2. Provides only relevant tools and documentation
3. Makes workflows data-driven (YAML), not improvised
4. Pins parameters in files, not in agent memory
5. Produces deterministic, auditable artifacts

## Target Users

1. **Video Producer (Primary)**: Uses CLI to run workflows, reviews generated artifacts
2. **AI Coding Agent (Secondary)**: Implements and extends the framework using TDD
3. **Future Web UI Users**: Fill in mad-lib forms, upload files, trigger workflows

## Functional Requirements

### FR-1: Workflow Definition (YAML)
- Versioned schema for forward compatibility
- Variables with explicit defaults
- Ordered steps with unique IDs
- Support for ensure_dirs, write_file, split_sections, run_command, llm_generate

### FR-2: CLI Runner
- `vwf run <workflow.yaml> --workdir <dir> [--var k=v]* [--dry-run] [--allow <cmd>]*`
- Dry-run prints planned operations without mutations
- Real run produces artifacts + run.json manifest

### FR-3: Template Rendering
- `{{var}}` substitution in file content and paths
- Error on missing variables (no silent defaults)

### FR-4: Step Execution
- **ensure_dirs**: Create directories relative to workdir
- **write_file**: Render template, write to path
- **split_sections**: Extract sections by heading markers
- **run_command**: Execute shell command (allowlist enforced)
- **llm_generate**: Call LLM (mock or real), write response

### FR-5: Run Manifest (run.json)
- Workflow name/version
- Variables used
- Step reports (id, status, duration, artifacts)
- Environment snapshot (timestamps, tool versions)

### FR-6: Command Safety
- Commands require explicit `--allow` flag
- Unknown commands fail with remediation message
- Stdout/stderr captured to artifacts

### FR-7: LLM Integration
- Trait-based abstraction (`LlmClient`)
- MockLlmClient for tests (deterministic responses)
- Future: Claude Code CLI adapter behind feature flag

### FR-8: Web UI (Future)
- List available workflows
- Mad-lib form for variables
- File upload for inputs
- Generate RunRequest JSON
- Call local HTTP runner (future)

## Non-Functional Requirements

### NFR-1: Determinism
- Stable output paths (no random names)
- Reproducible runs given same inputs

### NFR-2: Auditability
- Full provenance in run.json
- Diff-able across runs

### NFR-3: Testability
- Unit tests with no I/O (mock Runtime)
- Integration tests with temp directories

### NFR-4: Explainable Errors
- Errors include: step id, field, expected value, remediation

### NFR-5: Safety
- No implicit command execution
- Allowlist prevents accidental destructive commands

## Out of Scope (V1)

- Real-time video processing
- Cloud deployment
- Multi-user collaboration
- Video rendering/encoding (delegated to external tools)

## Success Metrics

1. Zero "agent drift" errors when using framework
2. 100% test coverage on core engine
3. Sub-second dry-run for typical workflows
4. Clear, actionable error messages for all validation failures

## Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| M1 | Workflow runner (no shell, no LLM) | In Progress |
| M2 | Shell step with allowlist | Planned |
| M3 | LLM adapter layer | Planned |
| M4 | Web UI (Yew/WASM) | Planned |
