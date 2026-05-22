# `RIOTBOX-928` Extract W-30 resample diagnostics label helpers

- Ticket: `RIOTBOX-928`
- Title: `Extract W-30 resample diagnostics label helpers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-928/extract-w-30-resample-diagnostics-label-helpers`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-928-w30-resample-diagnostics-labels`
- Linear branch: `feature/riotbox-928-extract-w-30-resample-diagnostics-label-helpers`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#921 (https://github.com/marang/riotbox/pull/921)`
- Merge commit: `0f5943cc7329f0e86d4598f69c0232cd6dd1da39`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; git diff --check main...HEAD; scripts/run_compact.sh /tmp/riotbox-928-just-ci.log just ci; GitHub Actions Rust CI run 26290687144 passed`
- Docs touched: `docs/archive/linear_issues/RIOTBOX-928.md; docs/archive/linear_issues/2026-05.md; docs/archive/linear_issues/index.md`
- Follow-ups: `RIOTBOX-929 reviews current P012 real-source timing confidence rows`

## Why This Ticket Existed

The P015 TUI ownership review identified the remaining W-30 resample tap/source/lineage/mix helpers as a cohesive diagnostics label cluster.

## What Shipped

- Extracted W-30 resample tap, lineage, route, source, focus, active-state, and mix compact labels into diagnostics_mc202_w30_logs/w30_resample.rs while preserving rendering behavior.

## Notes

- Helper ownership split only; no ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
