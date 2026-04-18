# `RIOTBOX-134` Add one bounded Scene Brain baseline for the queued help and footer cue stack

- Ticket: `RIOTBOX-134`
- Title: `Add one bounded Scene Brain baseline for the queued help and footer cue stack`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-134/add-one-bounded-scene-brain-baseline-for-the-queued-help-and-footer`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-134-scene-guidance-baseline`
- Linear branch: `feature/riotbox-134-add-one-bounded-scene-brain-baseline-for-the-queued-help-and`
- Assignee: `Markus`
- Labels: `None`
- PR: `#126`
- Merge commit: `85ae5792f5550105b84bd86749f971c8d64246d0`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#345`
- Docs touched: `docs/README.md`, `docs/benchmarks/README.md`, `docs/benchmarks/scene_guidance_stack_baseline_2026-04-18.md`
- Follow-ups: `RIOTBOX-135`

## Why This Ticket Existed

The current Scene Brain seam had already spread across `Jam`, `Help`, and `Log`, but the repo only measured isolated readability cues. Riotbox needed one bounded benchmark artifact that treats the queued-scene guidance stack as one small learning seam before that path drifts further.

## What Shipped

- added a new `scene_guidance_stack_baseline_2026-04-18.md` artifact under `docs/benchmarks/`
- recorded the current `Footer -> Help -> Log` guidance flow for queued `jump` and `restore`
- updated the benchmark and docs indexes to include the new guidance-stack artifact

## Notes

- this slice was benchmark-only; it did not change runtime or shell behavior
- the baseline stays manual and repo-local, which matches the current maturity of Scene Brain UX benchmarking
