# `RIOTBOX-927` Extract first-run capture routing UI helpers

- Ticket: `RIOTBOX-927`
- Title: `Extract first-run capture routing UI helpers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-927/extract-first-run-capture-routing-ui-helpers`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-927-first-run-capture-routing`
- Linear branch: `feature/riotbox-927-extract-first-run-capture-routing-ui-helpers`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#920 (https://github.com/marang/riotbox/pull/920)`
- Merge commit: `0b1293d08f2719366ee16646144b89116b2fa977`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; git diff --check main...HEAD; scripts/run_compact.sh /tmp/riotbox-927-just-ci.log just ci; GitHub Actions Rust CI run 26290334183 passed`
- Docs touched: `docs/archive/linear_issues/RIOTBOX-927.md; docs/archive/linear_issues/2026-05.md; docs/archive/linear_issues/index.md`
- Follow-ups: `RIOTBOX-928 continues P015 W-30 resample diagnostics label extraction`

## Why This Ticket Existed

The P015 review identified first_run_capture.rs as mixing first-run staging with capture routing detail.

## What Shipped

- Extracted capture routing lines and heard-path labeling into ui/first_run_capture/routing.rs while preserving the existing first_run_capture::capture_routing_lines surface.

## Notes

- Helper ownership split only; no ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
