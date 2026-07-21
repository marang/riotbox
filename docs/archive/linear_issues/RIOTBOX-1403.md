# `RIOTBOX-1403` P023: Stop fixture demo-bank evidence from driving live readiness priorities

- Ticket: `RIOTBOX-1403`
- Title: `P023: Stop fixture demo-bank evidence from driving live readiness priorities`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1403/p023-stop-fixture-demo-bank-evidence-from-driving-live-readiness`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-21`
- Finished: `2026-07-21`
- Branch: `feature/riotbox-1403-p023-stop-fixture-demo-bank-evidence-from-driving-live`
- Linear branch: `feature/riotbox-1403-p023-stop-fixture-demo-bank-evidence-from-driving-live`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `benchmark`
- PR: `#1373 (https://github.com/marang/riotbox/pull/1373)`
- Merge commit: `637e9355bb1d94e60d38fee1d2f909d02d870064`
- Deleted from Linear: `2026-07-21`
- Verification: `just ci; focused demo-bank/readiness fixture suites; Python byte-compilation; git diff --check; GitHub rust-ci`
- Docs touched: `docs/README.md; docs/execution_roadmap.md; docs/phase_definition_of_done.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1405 proves the exact weak/bad-timing degraded or reject product path with structured human review.`

## Why This Ticket Existed

Prevent checked-in calibration fixtures from being counted as live musician evidence and hiding missing Golden Path review.

## What Shipped

- Separated live_readiness from fixture_calibration, required typed hash-matched human provenance for live verdicts, bound queues/packs to bank identity, and modeled reviewed degraded/unavailable/reject family success without fallback music.

## Notes

- Maintenance/regression only; no audio-producing behavior changed and no musical-progress claim was made.
