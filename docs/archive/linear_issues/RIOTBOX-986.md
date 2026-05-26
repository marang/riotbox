# `RIOTBOX-986` Expose capture target boundary and length in Capture screen

- Ticket: `RIOTBOX-986`
- Title: `Expose capture target boundary and length in Capture screen`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-986/expose-capture-target-boundary-and-length-in-capture-screen`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-986-expose-capture-target-boundary-and-length-in-capture-screen`
- Linear branch: `feature/riotbox-986-expose-capture-target-boundary-and-length-in-capture-screen`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#978 (https://github.com/marang/riotbox/pull/978)`
- Merge commit: `f94351da32821d37682c0f022add58c8e9abc4e2`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-986-rebased-capture-target.log cargo test -p riotbox-app renders_capture_target -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-986-rebased-capture-context.log cargo test -p riotbox-app renders_capture_shell_snapshot_with_capture_context -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-986-rebased-ci.log just ci`
- Docs touched: `docs/jam_recipes.md`, `docs/research_decision_log.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make the Capture screen explain what c will queue before the musician presses it, without requiring inference from Source Map or observer diagnostics.

## What Shipped

- Added compact Capture screen target label derived from runtime capture length intent and Source Map readiness/range projection.
- Rendered trusted/user-confirmed timing, untrusted listen-first timing, and phrase fallback labels.
- Reused the same label in the first capture Do Next prompt with focused render coverage.

## Notes

- Rebased onto current main after RIOTBOX-985 merged, retargeted PR #978 to main, then reran focused Capture screen tests and just ci.
