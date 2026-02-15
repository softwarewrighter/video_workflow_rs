# Git Submodule Consideration: video-publishing

## Current State

- `video-publishing/tools/` contains Rust binaries: vid-image, vid-concat, vid-avatar, vid-lipsync, vid-composite, vid-volume
- VWF calls these via `run_command` with paths like `~/github/softwarewrighter/video-publishing/tools/target/release/vid-concat`
- Hard-coded paths are brittle

## Options

### Option A: Git Submodule

```bash
cd video_workflow_rs
git submodule add git@github.com:softwarewrighter/video-publishing.git deps/video-publishing
```

**Pros:**
- Tools versioned with VWF
- Clone includes everything needed
- CI/CD can build tools

**Cons:**
- Submodule complexity
- Need to keep in sync
- Larger repo

**Usage:**
```yaml
# In workflow
run_command:
  program: deps/video-publishing/tools/target/release/vid-concat
  args: [...]
```

### Option B: Workspace Member

Move `video-publishing/tools` crates into VWF workspace:

```
video_workflow_rs/
├── crates/
│   ├── vwf-core/
│   ├── vwf-cli/
│   ├── vwf-web/
│   ├── vid-concat/      # Moved from video-publishing
│   ├── vid-image/
│   ├── vid-avatar/
│   └── ...
```

**Pros:**
- Single workspace
- Shared dependencies
- Simpler CI/CD

**Cons:**
- Breaks video-publishing repo
- May want tools standalone

### Option C: Install to ~/.local/bin

Use `sw-install` to install vid-* tools globally:

```bash
sw-install -p ~/github/softwarewrighter/video-publishing/tools/vid-concat
```

**Pros:**
- Tools available everywhere
- No path hardcoding in workflows
- Already supported

**Cons:**
- Manual installation step
- Version management unclear

**Usage:**
```yaml
# In workflow
run_command:
  program: vid-concat  # Found in PATH
  args: [...]
```

### Option D: Configuration File

Define tool paths in a config file:

```toml
# ~/.config/vwf/tools.toml
[tools]
vid-concat = "~/github/softwarewrighter/video-publishing/tools/target/release/vid-concat"
vid-image = "..."
ffmpeg = "/opt/homebrew/bin/ffmpeg"
rsvg-convert = "/opt/homebrew/bin/rsvg-convert"
playwright-cli = "npx playwright"
```

**Pros:**
- Flexible per-machine
- No repo changes needed
- Supports different installations

**Cons:**
- Another config file
- Must set up on each machine

## Recommendation

**Start with Option D (Configuration)**, consider **Option A (Submodule)** later.

Rationale:
1. Config file is quick to implement
2. Doesn't require repo restructuring
3. Supports different machines (dev laptop vs. render farm)
4. Can add submodule later if needed for CI/CD

## Implementation

```rust
// crates/vwf-core/src/tools.rs

struct ToolConfig {
    tools: HashMap<String, PathBuf>,
}

impl ToolConfig {
    fn load() -> Result<Self> {
        // 1. Check ~/.config/vwf/tools.toml
        // 2. Check ./vwf-tools.toml (project-local)
        // 3. Fall back to PATH lookup
    }

    fn resolve(&self, tool: &str) -> Result<PathBuf> {
        self.tools.get(tool)
            .cloned()
            .or_else(|| which::which(tool).ok())
            .ok_or_else(|| anyhow!("Tool not found: {tool}"))
    }
}
```

```yaml
# In workflow
tasks:
  concat:
    kind: run_command
    tool: vid-concat  # Resolved via ToolConfig
    args: [--list, clips.txt, --output, final.mp4]
```
