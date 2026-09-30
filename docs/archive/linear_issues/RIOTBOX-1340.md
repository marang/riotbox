# `RIOTBOX-1340` P023: Replace runtime glob imports with explicit audio ownership imports

- Ticket: `RIOTBOX-1340`
- Title: `P023: Replace runtime glob imports with explicit audio ownership imports`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1340/p023-replace-runtime-glob-imports-with-explicit-audio-ownership`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1340-explicit-runtime-imports`
- Linear branch: `feature/riotbox-1340-p023-replace-runtime-glob-imports-with-explicit-audio`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1559 (https://github.com/marang/riotbox/pull/1559)`
- Merge commit: `a7195dcb6f95cfefe0017c3e0ba092af8789d8a1`
- Deleted from Linear: `2026-09-30`
- Verification: `Full just ci; 279 audio library tests passed, one ignored; 369 test-result entries unchanged; all non-import tokens identical in ten Rust files; native CI 36778070730 both jobs green; sequential solo Rust review, no outstanding findings.`
- Docs touched: `docs/reviews/riotbox_1340_explicit_runtime_imports_2026-09-30.md`
- Follow-ups: `Runtime test include migration remains in the existing inventory; no new task required.`

## Why This Ticket Existed

Make Runtime audio dependencies explicit without changing callback behavior.

## What Shipped

- Nine production wildcard imports replaced with semantic-owner imports; parent aggregation pruned while compatibility and test bridges remain.

## Notes

- No DSP, audio policy, public API, source access, playback or device validation change. Import-only slice does not increment architecture cadence.
