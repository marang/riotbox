# `RIOTBOX-976` Add capture length intent controls for source windows

- Ticket: `RIOTBOX-976`
- Title: `Add capture length intent controls for source windows`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-976/add-capture-length-intent-controls-for-source-windows`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-976-add-capture-length-intent-controls-for-source-windows`
- Linear branch: `feature/riotbox-976-add-capture-length-intent-controls-for-source-windows`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#967 (https://github.com/marang/riotbox/pull/967)`
- Merge commit: `dfc7eee4dace3b49bc58d1028d233282aed00a11`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-976-rebased-capture-length.log cargo test -p riotbox-app capture_length -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-976-rebased-phrase.log cargo test -p riotbox-app phrase_capture_length -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-976-rebased-ci.log just ci`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make capture length a deliberate musical intent instead of a hidden fixed window, while preserving replay/restore truth for source-window decisions.

## What Shipped

- Added typed CaptureLengthIntent, capture.set_length, and Session runtime capture state.
- Added immediate capture length controls and observer/Capture-screen status.
- Resolved source-window duration from capture intent when capture.bar_group omits explicit bars, with replay support.

## Notes

- Rebased onto current main after RIOTBOX-975 merged, retargeted PR #967 to main, then reran local focused tests and just ci.
