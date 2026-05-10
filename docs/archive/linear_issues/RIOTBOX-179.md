# `RIOTBOX-179` Add a focused Capture readability baseline after Do Next hierarchy

- Ticket: `RIOTBOX-179`
- Title: `Add a focused Capture readability baseline after Do Next hierarchy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-179/add-a-focused-capture-readability-baseline-after-do-next-hierarchy`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-179-capture-readability-baseline`
- PR: `#169`
- Merge commit: `1c4c60c`
- Labels: `benchmark`, `ux`, `TUI`
- Follow-ups: `RIOTBOX-180`

## Why This Ticket Existed

After the Capture screen gained a clearer `Do Next` hierarchy, the repo needed a stable readability reference before additional Capture help or layout changes could drift the first-action path.

## What Shipped

- Added `docs/benchmarks/capture_do_next_readability_baseline_2026-04-25.md`.
- Recorded the expected scan order for `Do Next`, `hear ...`, `Provenance`, and `Advanced Routing`.
- Linked the new baseline from the benchmark index and top-level docs index.

## Notes

- This was docs-only and did not change TUI behavior, sampler behavior, or audio QA coverage.
