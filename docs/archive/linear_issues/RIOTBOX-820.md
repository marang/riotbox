# `RIOTBOX-820` Expose split beat-period evidence in Source Timing example report

- Ticket: `RIOTBOX-820`
- Title: `Expose split beat-period evidence in Source Timing example report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-820/expose-split-beat-period-evidence-in-source-timing-example-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-820-source-timing-beat-evidence-report`
- Linear branch: `feature/riotbox-820-expose-split-beat-period-evidence-in-source-timing-example`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#815 (https://github.com/marang/riotbox/pull/815)`
- Merge commit: `3b397581f485c97c975ba669683cb8bcd8999764`
- Verification: `GitHub Actions Rust CI #1984 passed; local just ci passed; source-timing example report fixtures/local gate passed; source-timing readiness report gate passed.`
- Docs touched: `docs/benchmarks/source_timing_example_probe_report_2026-05-11.md`
- Follow-ups: `Next report expansion should split helper responsibilities first because source_timing_example_probe_report.py is near the soft review budget.`

## Why This Ticket Existed

The Source Timing spec calls for beat-period evidence in QA reports, but the example report only exposed aggregate readiness and alternate evidence.

## What Shipped

- Added beat matched-onset ratio, median-distance ratio, beat alternate count, and downbeat phase alternate count to the Source Timing example report and expectation fixtures.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.
