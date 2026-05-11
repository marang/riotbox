# `RIOTBOX-768` Add real example Source Timing probe report gate

- Ticket: `RIOTBOX-768`
- Title: `Add real example Source Timing probe report gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-768/add-real-example-source-timing-probe-report-gate`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-768-add-real-example-source-timing-probe-report-gate`
- Linear branch: `feature/riotbox-768-add-real-example-source-timing-probe-report-gate`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#762 (https://github.com/marang/riotbox/pull/762)`
- Merge commit: `f594dd688407e9445f9a5279727826b15d2cc35d`
- Deleted from Linear: `2026-05-11`
- Verification: `just source-timing-example-probe-report-fixtures; missing-source report smoke; just source-timing-example-probe-report artifacts/audio_qa/local/source_timing_example_probe_report.md; git diff --check; GitHub Actions Rust CI passed on PR #762`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 real-example Source Timing checks were still ad hoc loops over local WAVs; the repo needed a repeatable report/gate that skips missing uncommitted audio cleanly.

## What Shipped

- Added scripts/source_timing_example_probe_report.py, Just recipes for local reports and fixture smoke coverage, a committed source_timing_probe JSON fixture, CI wiring through just ci, and a Source Timing spec note for the report seam.

## Notes

- None
