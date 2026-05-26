# `RIOTBOX-984` Align Source Map capture range with next capture boundary

- Ticket: `RIOTBOX-984`
- Title: `Align Source Map capture range with next capture boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-984/align-source-map-capture-range-with-next-capture-boundary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-984-align-source-map-capture-range-with-next-capture-boundary`
- Linear branch: `feature/riotbox-984-align-source-map-capture-range-with-next-capture-boundary`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#976 (https://github.com/marang/riotbox/pull/976)`
- Merge commit: `dddbc8ca70bd598524caaac94631a008c3b201f3`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-984-rebased-source-map.log cargo test -p riotbox-core source_map -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-984-rebased-capture.log cargo test -p riotbox-app renders_capture -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-984-rebased-pending.log cargo test -p riotbox-app capture_pending_do_next -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-984-rebased-ci.log just ci`
- Docs touched: `docs/jam_recipes.md`, `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Align the Source Map cap preview with the musical boundary where c source capture actually commits, instead of drawing from a floored current transport beat while queueing elsewhere.

## What Shipped

- Projected Source Map capture ranges from the next bar boundary after current Session transport position.
- Queued capture.bar_group at next_bar for the c source-capture path.
- Updated observer/UI expectations so capture range row and Capture pending cues share the same boundary contract.

## Notes

- Rebased onto current main after RIOTBOX-983 merged, retargeted PR #976 to main, then reran focused Source Map/Capture tests and just ci.
