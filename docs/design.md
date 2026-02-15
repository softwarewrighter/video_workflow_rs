# Design Document

## Design Philosophy

### Core Principle: Workflows are Data

The fundamental insight is that AI agent drift happens because workflows exist as "remembered instructions" rather than explicit data. By making workflows YAML files, we:

1. **Pin parameters**: No improvisation of values
2. **Enable validation**: Fail fast with actionable errors
3. **Support diffing**: Compare runs to detect drift
4. **Allow versioning**: Git tracks workflow changes

### Inversion of Control

Instead of an agent deciding what to do next, the framework:
1. Reads explicit workflow definition
2. Executes steps in order
3. Provides only necessary context per step
4. Captures outputs deterministically

## Key Design Decisions

### D1: Runtime Trait for Testability

**Decision**: All side effects (filesystem, commands, LLM) go through traits.

**Rationale**:
- Unit tests run without I/O
- Dry-run mode is trivial to implement
- LLM calls can be mocked with deterministic responses

**Implementation**:
```rust
pub trait Runtime {
    fn create_dir(&self, path: &Path) -> Result<()>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn read_file(&self, path: &Path) -> Result<String>;
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput>;
}

pub trait LlmClient {
    fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;
}
```

### D2: Step ID in All Errors

**Decision**: Every error message includes the step ID that caused it.

**Rationale**:
- Workflows can have many steps
- "Unknown step type" is useless without context
- "Step 'write_template_prompt': unknown type 'writ_file'" is actionable

**Implementation**:
```rust
#[derive(Error, Debug)]
pub enum StepError {
    #[error("Step '{step_id}': {message}")]
    Validation { step_id: String, message: String },
}
```

### D3: No Magic Defaults

**Decision**: If a parameter affects output, it must be explicit.

**Rationale**:
- Implicit defaults cause hard-to-debug differences between runs
- Explicit is better than implicit (Zen of Python applies to Rust too)
- Makes workflow YAML self-documenting

**Examples**:
- Working directory: Always specified via `--workdir`
- Variables: Must be declared in `vars:` section
- Commands: Must be in allowlist

### D4: Template Syntax

**Decision**: Use simple `{{var}}` syntax, not a full templating engine.

**Rationale**:
- Conditionals and loops add complexity and footguns
- If complex logic needed, use a separate step
- Simple substitution covers 90% of use cases

**Future**: Can adopt a crate like `handlebars` if needed, but start minimal.

### D5: Allowlist for Commands

**Decision**: Shell commands require explicit `--allow <cmd>` flags.

**Rationale**:
- Prevents accidental `rm -rf` or similar disasters
- Makes workflow dependencies explicit
- Supports audit/compliance requirements

**Implementation**:
```bash
vwf run workflow.yaml --workdir work --allow ffmpeg --allow vhs
```

### D6: Manifest-First Reporting

**Decision**: Every run produces a `run.json` manifest, even on failure.

**Rationale**:
- Partial progress is still valuable
- Failed steps have status and error messages
- Enables post-mortem analysis

**Schema**:
```json
{
  "workflow": "shorts_narration",
  "version": 1,
  "started_at": "2024-01-15T10:30:00Z",
  "completed_at": "2024-01-15T10:30:05Z",
  "vars": {"project_name": "Demo"},
  "steps": [
    {"id": "ensure_dirs", "status": "ok", "duration_ms": 12},
    {"id": "write_template", "status": "ok", "duration_ms": 5, "artifacts": ["work/prompt.txt"]}
  ]
}
```

## Module Design

### vwf-core

```
vwf-core/
|-- src/
|   |-- lib.rs          # Public API exports
|   |-- config.rs       # WorkflowConfig, StepConfig parsing
|   |-- engine.rs       # Runner, RunReport
|   |-- render.rs       # {{var}} template substitution
|   |-- runtime.rs      # Runtime trait, FsRuntime, DryRunRuntime
|   |-- steps.rs        # Step implementations
```

### Step Configuration (Enum Pattern)

```rust
#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum StepConfig {
    #[serde(rename = "ensure_dirs")]
    EnsureDirs { id: String, dirs: Vec<String> },

    #[serde(rename = "write_file")]
    WriteFile { id: String, path: String, content: String },

    #[serde(rename = "llm_generate")]
    LlmGenerate {
        id: String,
        system: Option<String>,
        user_prompt_path: String,
        output_path: String,
        provider: String,
        mock_response: Option<String>,
    },
    // ...
}
```

### Error Handling Strategy

1. **Parse errors**: Include YAML line numbers when possible
2. **Validation errors**: Step ID + field + expected value + remediation
3. **Runtime errors**: Step ID + operation + underlying error
4. **Never panic**: Return `Result<T, Error>` throughout

## Integration Patterns

### CLI Integration

```
vwf-cli
   |
   +--> vwf-core::config::parse_workflow(yaml_str)
   +--> vwf-core::engine::Runner::new(config, runtime)
   +--> runner.run() --> RunReport
   +--> write run.json
```

### Future Web UI Integration

```
vwf-web (Yew)
   |
   +--> User fills form (mad-lib)
   +--> Generate RunRequest JSON
   +--> POST to local HTTP runner
   +--> Display results
```

## Testing Strategy

### Unit Tests (vwf-core)

- Parse valid/invalid YAML
- Template rendering (success and missing var)
- Step validation errors
- Step execution with mock runtime

### Integration Tests

- Full workflow execution with temp directory
- Dry-run vs real-run comparison
- Command allowlist enforcement

### Property Tests (Future)

- Arbitrary workflows always produce valid manifest
- Template rendering is idempotent

## Performance Considerations

- Workflows are small (< 1000 steps typically)
- No streaming needed for V1
- Template rendering is O(n) in content size
- LLM latency dominates (external dependency)

## Security Considerations

- YAML parsing: Use `serde_yaml` (safe by default)
- Command execution: Allowlist-only
- Path traversal: Validate paths stay within workdir
- Secrets: Never log or manifest variable values (future: redact option)
