# Run Request (future)

This repo intentionally runs workflows directly from YAML.

For "UI -> runner" integration, a `RunRequest` JSON object is useful:

```json
{
  "workflow_path_or_text": "...",
  "workdir": "work/demo",
  "vars": {"project_name":"...", "audience":"..."},
  "inputs": [
    {"src":"~/clips/obs.mp4", "dest":"inputs/obs.mp4"}
  ],
  "allow_commands": ["ffmpeg", "vhs"]
}
```

A future crate can:
- validate this request
- stage inputs into workdir
- run the workflow
- return a report for the UI
