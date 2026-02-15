# Flow Charts

## 5MLC Shorts Workflow DAG

```
                    +------------------+
                    |  episode_number  |
                    |    concepts[]    |
                    +--------+---------+
                             |
                             v
                    +------------------+
                    | generate_outline |
                    +--------+---------+
                             |
              +--------------+--------------+
              |              |              |
              v              v              v
     +------------+  +-------------+  +---------------+
     | narration  |  |   slides    |  |  description  |
     |    .txt    |  |  outline    |  |     .txt      |
     +------+-----+  +------+------+  +---------------+
            |               |
            v               |
     +------------+         |
     |   REVIEW   |<--------+
     | CHECKPOINT |
     +------+-----+
            |
            +------------------+
            |                  |
            v                  v
     +-------------+    +-------------+
     | split into  |    | create HTML |
     | TTS scripts |    |   slides    |
     +------+------+    +------+------+
            |                  |
            |                  v
            |           +-------------+
            |           |  background |...(optional)
            |           |    image    |
            |           +------+------+
            |                  |
            v                  v
     +-------------+    +-------------+
     |   TTS gen   |    |  screenshot |
     | (sequential)|    |  (parallel) |
     +------+------+    +------+------+
            |                  |
            v                  v
     +-------------+    +-------------+
     |   whisper   |    |    PNG      |
     |   verify    |    |   images    |
     +------+------+    +------+------+
            |                  |
            v                  |
     +-------------+           |
     |   REVIEW    |           |
     |  TTS AUDIO  |           |
     +------+------+           |
            |                  |
            +--------+---------+
                     |
                     v
              +-------------+
              | create video|
              |    clips    |
              +------+------+
                     |
     +---------------+---------------+
     |               |               |
     v               v               v
+----------+   +----------+   +----------+
|  intro   |   |  slides  |   |  outro   |
|  (opt)   |   |  clips   |   |  (opt)   |
+----+-----+   +----+-----+   +----+-----+
     |              |              |
     |   +----------+----------+   |
     |   |                     |   |
     +---+---------------------+---+
                   |
                   v
            +-------------+
            |  vid-concat |
            | (reencode)  |
            +------+------+
                   |
                   v
            +-------------+
            | vid-volume  |
            | (normalize) |
            +------+------+
                   |
                   v
            +-------------+
            |  final.mp4  |
            +-------------+
```

## Progressive Preview Assembly

```
State 1: Text Only
+--------------------------------------------------+
| [x] narration.txt     Generated                   |
| [x] slides-outline    Generated                   |
| [ ] background        NOT PROVIDED                |
| [ ] TTS audio         BLOCKED (waiting review)    |
| [ ] screenshots       BLOCKED (no background)     |
| [ ] video clips       BLOCKED (no audio/images)   |
+--------------------------------------------------+
Preview: None available yet


State 2: After Review + Background Added
+--------------------------------------------------+
| [x] narration.txt     Generated                   |
| [x] slides-outline    Generated                   |
| [x] background        PROVIDED                    |
| [ ] TTS audio         RUNNING (3/7 complete)      |
| [x] screenshots       COMPLETE (7/7)              |
| [ ] video clips       PARTIAL (0/7)               |
+--------------------------------------------------+
Preview: Slideshow (images only, no audio)


State 3: TTS Complete
+--------------------------------------------------+
| [x] narration.txt     Generated                   |
| [x] slides-outline    Generated                   |
| [x] background        PROVIDED                    |
| [x] TTS audio         COMPLETE (7/7)              |
| [x] screenshots       COMPLETE (7/7)              |
| [~] video clips       RUNNING (4/7 complete)      |
+--------------------------------------------------+
Preview: Partial video (4 clips + placeholders)


State 4: Missing Intro/Outro
+--------------------------------------------------+
| [x] narration.txt     Generated                   |
| [x] background        PROVIDED                    |
| [x] TTS audio         COMPLETE                    |
| [x] screenshots       COMPLETE                    |
| [x] slide clips       COMPLETE (7/7)              |
| [ ] intro             NOT PROVIDED (skipped)      |
| [ ] outro             NOT PROVIDED (skipped)      |
+--------------------------------------------------+
Preview: Full video without intro/outro


State 5: All Complete
+--------------------------------------------------+
| [x] All artifacts     COMPLETE                    |
| [x] intro             PROVIDED & processed        |
| [x] outro             PROVIDED & processed        |
+--------------------------------------------------+
Preview: final.mp4 (ready for upload)
```

## Resource Contention Handling

```
Time -->
        t0      t1      t2      t3      t4      t5      t6

GPU     +-------+       +-------+       +-------+
        | TTS-1 |       | TTS-2 |       | TTS-3 |
        +-------+       +-------+       +-------+

CPU-1   +---------------+       +---------------+
        | screenshot-1  |       | screenshot-4  |
        +---------------+       +---------------+

CPU-2           +---------------+       +---------------+
                | screenshot-2  |       | screenshot-5  |
                +---------------+       +---------------+

CPU-3                   +---------------+       +-------+
                        | screenshot-3  |       | concat|
                        +---------------+       +-------+

Legend:
- TTS runs sequentially (GPU constraint)
- Screenshots run in parallel (no constraint)
- Concat waits for all clips
```

## Checkpoint Flow

```
                 +------------------+
                 | Generate Content |
                 +--------+---------+
                          |
                          v
                 +------------------+
                 |    CHECKPOINT    |
                 |   "Review Text"  |
                 +--------+---------+
                          |
          +---------------+---------------+
          |                               |
          v                               v
   +--------------+                +--------------+
   |   APPROVE    |                |    REJECT    |
   +--------------+                +--------------+
          |                               |
          v                               v
   +--------------+                +--------------+
   | Continue to  |                | User edits   |
   | next phase   |                | text files   |
   +--------------+                +------+-------+
                                          |
                                          v
                                   +--------------+
                                   | Regenerate   |
                                   | affected     |
                                   +------+-------+
                                          |
                                          +---> back to CHECKPOINT
```

## Artifact Dependency Graph

```
concepts.txt ----+
                 |
episode_num -----+---> generate_outline --+--> narration.txt
                                          |
                                          +--> slides-outline.txt
                                          |
                                          +--> description.txt

narration.txt -----> split_scripts -----> scripts/*.txt

scripts/01.txt ---+
                  +--> tts_generate --> audio/01.wav
reference.wav ----+

slides-outline.txt --+
                     +--> create_slides --> slides/01.html
background.jpg ------+

slides/01.html -----> screenshot -----> images/01.png

images/01.png ---+
                 +--> create_clip -----> clips/01.mp4
audio/01.wav ----+

clips/*.mp4 -------> concat ---------> combined.mp4

combined.mp4 ------> normalize ------> final.mp4
```

## Invalidation Cascade

```
User changes: background.jpg

        background.jpg (CHANGED)
               |
               v
        slides/01.html (INVALIDATE)
        slides/02.html (INVALIDATE)
        ...
               |
               v
        images/01.png (INVALIDATE)
        images/02.png (INVALIDATE)
        ...
               |
               v
        clips/01.mp4 (INVALIDATE)
        clips/02.mp4 (INVALIDATE)
        ...
               |
               v
        combined.mp4 (INVALIDATE)
               |
               v
        final.mp4 (INVALIDATE)

NOT invalidated:
  - narration.txt (no dependency on background)
  - audio/*.wav (no dependency on background)
  - scripts/*.txt (no dependency on background)
```
