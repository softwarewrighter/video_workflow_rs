# Rules Engine Concept: Declarative Production Rules

## Alternative Approach

Instead of explicit task DAG, define **production rules** that fire when conditions are met. The system continuously evaluates rules and executes matching ones.

This is similar to Prolog, CLIPS, or database triggers.

## Core Idea

```
WHEN <condition> THEN <action>
```

Rules are declarative. The engine decides what to run based on current state.

## Rule Definition

```yaml
rules:
  # Text generation
  - name: generate_narration
    when:
      - exists: inputs/concepts.txt
      - not_exists: work/narration.txt
    then:
      action: llm_generate
      inputs: [inputs/concepts.txt]
      outputs: [work/narration.txt, work/slides-outline.txt]

  # TTS generation (one at a time)
  - name: generate_tts_segment
    when:
      - exists: work/scripts/*.txt
      - for_each: $script in work/scripts/*.txt
      - not_exists: work/audio/${script.stem}.wav
      - not_running: tts_*  # Only one TTS at a time
    then:
      action: tts_generate
      inputs: [$script]
      outputs: [work/audio/${script.stem}.wav]
      locks: [gpu, tts]

  # Screenshot capture (parallel OK)
    - name: capture_screenshot
    when:
      - exists: showcase/slides/*.html
      - for_each: $slide in showcase/slides/*.html
      - not_exists: work/images/${slide.stem}.png
      - newer: $slide than work/images/${slide.stem}.png  # Or doesn't exist
    then:
      action: playwright_screenshot
      inputs: [$slide]
      outputs: [work/images/${slide.stem}.png]

  # Video clip creation
  - name: create_video_clip
    when:
      - exists: work/images/$name.png
      - exists: work/audio/$name.wav
      - not_exists: work/clips/$name.mp4
    then:
      action: create_clip
      inputs: [work/images/$name.png, work/audio/$name.wav]
      outputs: [work/clips/$name.mp4]

  # Preview generation (always runs when any clip changes)
  - name: generate_preview
    when:
      - any_changed: work/clips/*.mp4
    then:
      action: concat_preview
      inputs: [work/clips/*.mp4]
      outputs: [work/preview.mp4]
```

## Rule Evaluation Loop

```
while true:
    state = scan_filesystem()

    for rule in rules:
        if rule.conditions_met(state):
            if not rule.locks_available():
                continue  # Can't run yet, resource busy

            schedule(rule.action)

    if no_rules_fired and no_tasks_running:
        if checkpoints_pending:
            wait_for_user()
        else:
            break  # Workflow complete

    sleep(poll_interval)
```

## Condition Types

```yaml
conditions:
  # File existence
  exists: path/to/file
  not_exists: path/to/file

  # File comparison
  newer: file_a than file_b
  older: file_a than file_b
  checksum_changed: file since last_run

  # Pattern matching
  for_each: $var in pattern/*.ext
  any: pattern/*.ext
  all: pattern/*.ext

  # Resource locks
  not_running: pattern_*
  lock_available: resource_name

  # User approval
  checkpoint_approved: checkpoint_name

  # Time-based
  older_than: file, duration
```

## Advantages of Rules Engine

| DAG-based | Rules-based |
|-----------|-------------|
| Explicit dependencies | Implicit from conditions |
| Static structure | Dynamic, reactive |
| Must define all tasks | Rules infer what's needed |
| Change = re-run task | Change = rules re-evaluate |
| Good for fixed pipelines | Good for evolving workflows |

## Disadvantages

| DAG-based | Rules-based |
|-----------|-------------|
| Clear execution order | Non-obvious ordering |
| Easy to debug | Rule conflicts possible |
| Predictable | Emergent behavior |
| Explicit parallelism | Implicit parallelism |

## Hybrid Approach

Combine both: Define high-level phases as a DAG, use rules within phases.

```yaml
phases:
  - name: content_generation
    rules: [generate_narration, split_scripts]
    checkpoint: review_text

  - name: asset_creation
    rules: [create_slides, capture_screenshots, generate_tts]
    checkpoint: review_tts

  - name: video_assembly
    rules: [create_clips, add_music, concat_all]
    checkpoint: final_review
```

## When to Use Rules Engine

**Good fit:**
- Highly dynamic workflows
- Many independent producers/consumers
- Frequent additions of new rule types
- File-system watching scenarios

**Not a good fit:**
- Strict ordering requirements
- Complex multi-step dependencies
- Need for deterministic execution
- Audit/compliance requirements

## Recommendation

For VWF, the **DAG-based approach** is likely better because:
1. Video production has clear phase ordering
2. Dependencies are well-defined (TTS needs script, video needs audio)
3. Determinism is important (reproducible builds)
4. Easier to debug and explain

However, the **placeholder/progressive refinement** aspect borrows from rules thinking:
- "When background arrives, regenerate slides"
- "When any clip changes, regenerate preview"

This can be modeled as DAG with reactive edges rather than a full rules engine.
