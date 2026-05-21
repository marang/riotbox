# `RIOTBOX-899` Expose Source Timing phrase counts in probe and example reports

- Ticket: `RIOTBOX-899`
- Title: `Expose Source Timing phrase counts in probe and example reports`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-899/expose-source-timing-phrase-counts-in-probe-and-example-reports`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-899-source-timing-phrase-counts`
- Linear branch: `feature/riotbox-899-expose-source-timing-phrase-counts-in-probe-and-example`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`, `ux`
- PR: `#892 (https://github.com/marang/riotbox/pull/892)`
- Merge commit: `b26775e89752ff6616bbff6d79438efc4e02a6f2`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; python3 -m py_compile scripts/validate_source_timing_probe_json.py scripts/source_timing_example_probe_report.py scripts/source_timing_example_expectations.py; cargo test -p riotbox-core source_timing_probe_readiness_report; cargo test -p riotbox-audio --bin source_timing_probe; cargo test -p riotbox-audio --bin feral_grid_pack bpm_decision_tests; just source-timing-probe-json-validator-fixtures; just source-timing-example-probe-report-fixtures; just source-timing-example-probe-report-local; git diff --check; just ci; GitHub Rust CI #2219`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Probe and example-report reviewers could see phrase status but not the phrase-count evidence behind short-loop manual-confirm decisions.

## What Shipped

- Added primary phrase count and phrase bar count to Source Timing readiness reports.
- Carried the counts into source_timing_probe JSON/text output and validator fixtures.
- Added Phrase count and Phrase bars columns to the local Source Timing example probe report.
- Kept readiness, grid-use, lane behavior, and audio-output behavior unchanged.

## Notes

- None
