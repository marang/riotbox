# Audio QA Listening Review

Status: local human-review template  
Template date: `2026-04-26`

Use this file as the starting shape for local `notes.md` files beside generated audio QA artifacts.

Generated notes usually live under:

```text
artifacts/audio_qa/<date>/<pack-id>/<case-id>/notes.md
```

Do not commit local `notes.md` files by default. Commit only durable summaries or follow-up decisions under `docs/benchmarks/`, `docs/reviews/`, or Linear.

## Metadata

- Review date:
- Commit:
- Branch:
- Pack:
- Case:
- Source / fixture:
- Listener:
- Playback path:
- Listening environment:
- Artifact path:
- Observer path, if used:

## Automated Evidence

- Render command:
- Comparison command:
- Relevant tests:
- Metrics result: `pass` / `concern` / `fail`
- Baseline WAV:
- Candidate WAV:
- Comparison report:

## Listening Rubric

Use `1` to `4` for musical quality fields:

- `1`: unacceptable
- `2`: weak
- `3`: acceptable
- `4`: strong

Use `none`, `minor`, `major`, or `blocking` for artifact severity.

| Field | Score | Notes |
| --- | ---: | --- |
| Rhythmic clarity |  |  |
| Energy appropriateness |  |  |
| Transition quality |  |  |
| Variation usefulness |  |  |
| Support-layer tastefulness |  |  |
| Capture-worthiness |  |  |
| Artifact severity |  |  |

## What I Heard

- First impression:
- What changed from baseline to candidate:
- Best musical moment:
- Weak or annoying behavior:
- Timing feel:
- Repetition / sameness:
- Source retention or fallback suspicion:

## Verdict

- Result: `pass` / `concern` / `fail`
- Reason:
- Safe to merge from a listening perspective: `yes` / `no` / `not applicable`
- Needs new ticket: `no` / `RIOTBOX-`
- Durable summary to copy into repo docs or Linear:

## Follow-Up

- None:
- Ticket:
- Fixture or threshold to add:
- UX cue to clarify:
- Audio policy or render behavior to revisit:
