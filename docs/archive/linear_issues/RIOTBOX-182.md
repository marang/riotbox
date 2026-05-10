# `RIOTBOX-182` Add Capture pending-state readability baseline

- Ticket: `RIOTBOX-182`
- Title: `Add Capture pending-state readability baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-182/add-capture-pending-state-readability-baseline`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-182-capture-pending-readability-baseline`
- Linear branch: `feature/riotbox-182-add-capture-pending-state-readability-baseline`
- PR: `#172`
- Merge commit: `b3be194`
- Labels: `benchmark`, `ux`, `TUI`
- Follow-ups: `RIOTBOX-183`

## Why This Ticket Existed

After RIOTBOX-181 made Capture `Do Next` aware of pending capture-path intent, the repo needed a small readability baseline for that exact state so future Capture reductions do not regress the pending-first scan order.

## What Shipped

- Added `docs/benchmarks/capture_pending_do_next_readability_baseline_2026-04-25.md`.
- Recorded expected cues for queued capture, queued promotion, and W-30 reshape states.
- Linked the new baseline from `docs/benchmarks/README.md`.

## Verification

- `git diff --check main..HEAD`
- `just ci`

## Notes

- This was docs-only and did not change queue semantics, TUI behavior, audio output, or automated audio QA coverage.
