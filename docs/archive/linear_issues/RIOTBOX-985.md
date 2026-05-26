# `RIOTBOX-985` Apply capture length boundary semantics to committed source windows

- Ticket: `RIOTBOX-985`
- Title: `Apply capture length boundary semantics to committed source windows`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-985/apply-capture-length-boundary-semantics-to-committed-source-windows`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-985-apply-capture-length-boundary-semantics-to-committed-source-windows`
- Linear branch: `feature/riotbox-985-apply-capture-length-boundary-semantics-to-committed-source`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#977 (https://github.com/marang/riotbox/pull/977)`
- Merge commit: `91e472fbff4698c1fd2bd8826db028464c251df3`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-985-rebased-source-window.log cargo test -p riotbox-app source_window -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-985-rebased-capture-obs.log cargo test -p riotbox-app --bin riotbox-app observer_snapshot_records_committed_capture_source_window -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-985-rebased-ci.log just ci`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Add landed-capture observer evidence so QA can correlate visible cap preview, transport commit boundary, and the committed source window.

## What Shipped

- Added a top-level capture observer snapshot block for the latest capture.
- Exposed source-window provenance including source id, timing bounds, frame bounds, source-origin count, and creating action id.
- Tightened source-window tests to commit capture.bar_group at Bar boundaries instead of phrase-boundary assumptions.

## Notes

- Rebased onto current main after RIOTBOX-984 merged, retargeted PR #977 to main, then reran focused source-window/observer tests and just ci.
