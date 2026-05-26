# `RIOTBOX-983` Assert Source Map capture range in user-session observer probes

- Ticket: `RIOTBOX-983`
- Title: `Assert Source Map capture range in user-session observer probes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-983/assert-source-map-capture-range-in-user-session-observer-probes`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-983-assert-source-map-capture-range-in-user-session-observer`
- Linear branch: `feature/riotbox-983-assert-source-map-capture-range-in-user-session-observer`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#975 (https://github.com/marang/riotbox/pull/975)`
- Merge commit: `c8ed8b21373e1c4ef62060b2a23823faf329109d`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-983-rebased-feral-grid.log cargo test -p riotbox-app --bin user_session_observer_probe writes_feral_grid -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-983-rebased-probe-bin.log cargo test -p riotbox-app --bin user_session_observer_probe`; `scripts/run_compact.sh /tmp/riotbox-983-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Keep Source Map capture-range observer fields honest in the headless user-session probe path instead of only unit snapshots or terminal-render scraping.

## What Shipped

- Extended Feral-grid user-session observer probe tests to assert snapshot.source_map.
- Asserted capture range availability for locked/bar-grid timing.
- Asserted capture range remains unavailable for cautious/manual-review and fallback probe paths.

## Notes

- Rebased onto current main after RIOTBOX-982 merged, retargeted PR #975 to main, then reran observer probe tests and just ci.
