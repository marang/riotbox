# `RIOTBOX-996` Add local example anchor-evidence expectations

- Ticket: `RIOTBOX-996`
- Title: `Add local example anchor-evidence expectations`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-996/add-local-example-anchor-evidence-expectations`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-996-add-local-example-anchor-evidence-expectations`
- Linear branch: `feature/riotbox-996-add-local-example-anchor-evidence-expectations`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: `#988 (https://github.com/marang/riotbox/pull/988)`
- Merge commit: `56b4dd86ea900861276cdf7a5db0e0af9de0a85c`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-996-example-report-fixtures-rebased.log just source-timing-example-probe-report-fixtures`; `scripts/run_compact.sh /tmp/riotbox-996-example-report-local-rebased.log just source-timing-example-probe-report-local /tmp/riotbox-996-source-timing-report-rebased.md`; `git diff --check main..HEAD`; `scripts/run_compact.sh /tmp/riotbox-996-just-ci.log just ci`; `GitHub Actions Rust CI run 26449446349 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Tighten P012 real-source confidence proof so local example expectations verify compact anchor evidence, not only timing status and scores.

## What Shipped

- Added nested anchor_evidence expectation support for total, kick, backbeat, and transient anchor counts.
- Added invalid expectation fixtures for empty, negative, and unknown anchor evidence shapes plus mismatch coverage.
- Populated local example expectations with current anchor evidence counts, including Beat20 as transient-only 11/0/0/11.

## Notes

- No probe/analyzer, readiness policy, UI, Session, realtime audio, or output behavior changed.
