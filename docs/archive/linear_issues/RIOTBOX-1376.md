# `RIOTBOX-1376` P023: Reconcile shipped perform-risk cue in readiness

- Ticket: `RIOTBOX-1376`
- Title: `P023: Reconcile shipped perform-risk cue in readiness`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1376/p023-reconcile-shipped-perform-risk-cue-in-readiness`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1376-p023-reconcile-shipped-perform-risk-cue-in-readiness`
- Linear branch: `feature/riotbox-1376-p023-reconcile-shipped-perform-risk-cue-in-readiness`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1340 (https://github.com/marang/riotbox/pull/1340)`
- Merge commit: `46c9bf416e0ff9077d3af2fe0bc5a21f398d7cae`
- Deleted from Linear: `2026-07-02`
- Verification: `cargo test -p riotbox-app shell_state_jam_snapshot; cargo run -p riotbox-app --bin jam_perform_risk_cue_contract -- --output /tmp/riotbox-1376-contract.json; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1376-readiness; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-1375 shipped the Jam Trust bar/live? perform-risk cue, but P023 readiness still treated ui_cue as current until app evidence proved the shipped surface.

## What Shipped

- Added an app-emitted Jam perform-risk cue contract, wired P023 readiness to validate it, demoted ui_cue to stale regression control only when the contract passes, and advanced current evidence to fixture_threshold without allowing quality claims.

## Notes

- None
