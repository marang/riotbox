# `RIOTBOX-121` Measure scene action timing readability on the current perform-first shell

- Ticket: `RIOTBOX-121`
- Title: `Measure scene action timing readability on the current perform-first shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-121/measure-scene-action-timing-readability-on-the-current-perform-first`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-121-scene-timing-readability`
- Linear branch: `feature/riotbox-121-measure-scene-action-timing-readability-on-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#114`
- Merge commit: `b1a31b2161aa995b2842642b570c73376fadb8d6`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#321`
- Docs touched: `docs/benchmarks/README.md`, `docs/README.md`, `docs/benchmarks/scene_timing_readability_baseline_2026-04-18.md`
- Follow-ups: `RIOTBOX-122`, `RIOTBOX-124`

## Why This Ticket Existed

After the Scene Brain shell gained explicit boundary cues, pulse timing, and post-commit guidance, Riotbox needed one small benchmark baseline that records whether those changes actually make timing comprehension easier on `Jam`. Without a baseline, future timing-visibility changes would have no stable comparison point.

## What Shipped

- added the first `scene_timing_readability` benchmark baseline under `docs/benchmarks/`
- recorded the current Jam-shell path for reading queued boundary, pulse, and post-commit guidance without switching to `Log`
- indexed the new readability artifact in the benchmark and docs indexes

## Notes

- this slice was benchmark-only; it did not change UI or runtime behavior
- the baseline is still manual and repo-local, which is appropriate for the current stage of Scene Brain UX work
