# `RIOTBOX-129` Refresh scene timing readability benchmark after contrast, trail, and countdown cues

- Ticket: `RIOTBOX-129`
- Title: `Refresh scene timing readability benchmark after contrast, trail, and countdown cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-129/refresh-scene-timing-readability-benchmark-after-contrast-trail-and`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-129-scene-readability-refresh`
- Linear branch: `feature/riotbox-129-refresh-scene-timing-readability-benchmark-after-contrast`
- Assignee: `Markus`
- Labels: `None`
- PR: `#121`
- Merge commit: `35d8f86af424eae10e73511bfbfce76d7012375a`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#335`
- Docs touched: `docs/benchmarks/README.md`, `docs/benchmarks/scene_timing_readability_baseline_2026-04-18.md`
- Follow-ups: `RIOTBOX-130`, `RIOTBOX-134`

## Why This Ticket Existed

The first Scene Brain timing-readability artifact had been captured before the latest `Jam` and `Log` cues landed. Riotbox needed one benchmark-only follow-up that made the readability baseline truthful again after the contrast, trail, and countdown work changed what the shell actually shipped.

## What Shipped

- refreshed the Scene Brain timing-readability baseline against the current shell state
- recorded the shipped `live <> restore` contrast cue in `Jam`
- recorded the compact scene `trail ...` cue in `Log`
- updated the benchmark index summary to reflect that the artifact now covers timing, contrast, and recent-result readability together

## Notes

- this slice remained benchmark-only; it did not change UI or runtime behavior
- the artifact is still manual and repo-local on purpose, which fits the current state of Scene Brain UX measurement
