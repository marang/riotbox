# Observer / Audio QA Correlation

Status: local operator workflow template  
Template date: `2026-04-29`

Use this file when a Riotbox run needs both:

- control-path evidence from `riotbox-app --observer <events.ndjson>`
- output-path evidence from a generated audio QA `manifest.json`

Do not commit filled local correlation notes by default. Commit only durable conclusions under `docs/benchmarks/`, `docs/reviews/`, or Linear.

## Metadata

- Review date:
- Commit:
- Branch:
- Source / fixture:
- Recipe or user flow:
- Operator:
- Observer path:
- Audio QA manifest path:
- Audio QA pack:
- Listening environment:

## Commands

Observer run:

```bash
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat03_130BPM(Full).wav" --observer artifacts/audio_qa/local/user-session/events.ndjson
```

Audio output evidence:

```bash
just feral-grid-pack "data/test_audio/examples/Beat03_130BPM(Full).wav" local 130.0 8 1.0 0.0
```

Optional generated summary:

```bash
just observer-audio-correlate artifacts/audio_qa/local/user-session/events.ndjson artifacts/audio_qa/local/feral-grid-demo/manifest.json artifacts/audio_qa/local/observer_audio_summary.md
```

## Observer Evidence

- Observer schema:
- Launch mode:
- Audio runtime status:
- Key sequence:
- First queued action:
- First committed action:
- Commit boundary:
- Transport state after first commit:
- Render-state summary after first commit:
- Any rejected / ignored key outcomes:

## Audio Manifest Evidence

- Manifest schema version:
- Pack id:
- Result:
- BPM / bars / total frames:
- Artifact count:
- Full mix path:
- Full mix RMS:
- Full mix low-band RMS:
- MC-202 question/answer delta RMS, if relevant:
- Thresholds that matter for this flow:

## Correlation Check

| Question | Evidence | Result |
| --- | --- | --- |
| Did the expected keypress queue an action? |  | `yes` / `no` / `n/a` |
| Did the action commit on the expected boundary? |  | `yes` / `no` / `n/a` |
| Did the render-state projection match the intended lane? |  | `yes` / `no` / `n/a` |
| Did the audio manifest prove a non-silent output seam? |  | `yes` / `no` / `n/a` |
| Did the audio metrics prove the intended contrast? |  | `yes` / `no` / `n/a` |
| Is any mismatch likely user timing rather than engine output? |  | `yes` / `no` / `unclear` |

## Listening Notes

- What I expected to hear:
- What I actually heard:
- Strongest musical moment:
- Weakest or confusing moment:
- Timing / groove concern:
- Fallback suspicion:

## Verdict

- Result: `pass` / `concern` / `fail`
- Safe to trust this flow for current docs: `yes` / `no`
- Needs new ticket: `no` / `RIOTBOX-`
- Durable summary to copy into repo docs or Linear:
