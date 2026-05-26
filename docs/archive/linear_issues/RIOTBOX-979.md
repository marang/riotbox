# `RIOTBOX-979` Gate all-lane source consumers on confirmed timing state

- Ticket: `RIOTBOX-979`
- Title: `Gate all-lane source consumers on confirmed timing state`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-979/gate-all-lane-source-consumers-on-confirmed-timing-state`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-979-gate-all-lane-source-consumers-on-confirmed-timing-state`
- Linear branch: `feature/riotbox-979-gate-all-lane-source-consumers-on-confirmed-timing-state`
- Assignee: `Markus`
- Labels: `Feature`, `timing`
- PR: `#971 (https://github.com/marang/riotbox/pull/971)`
- Merge commit: `c1f451bc95caf81d9ce4ee2158ac02937dfcc458`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-979-rebased-readiness.log cargo test -p riotbox-core consumer_readiness -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-979-rebased-capture-unconfirmed.log cargo test -p riotbox-app manual_confirm_capture_does_not_materialize_source_window_until_confirmed -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-979-rebased-capture-confirmed.log cargo test -p riotbox-app user_confirmed_manual_grid_allows_capture_source_window -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-979-rebased-ci.log just ci`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prevent downstream source-window consumers from treating unconfirmed manual-confirm timing as bar-accurate capture/W-30 reuse truth just because tempo evidence exists.

## What Shipped

- Added shared SourceTimingConsumerReadiness in the core Jam view contract.
- Reused readiness in Source Map, observer confirmation matching, and capture source-window materialization.
- Gated capture source windows to analyzer-locked or matching user-confirmed timing with regressions for unconfirmed/manual/fallback cases.

## Notes

- Rebased onto current main after RIOTBOX-978 merged, retargeted PR #971 to main, then reran focused readiness/capture tests and just ci.
