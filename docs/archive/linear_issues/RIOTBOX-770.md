# `RIOTBOX-770` Add optional local example Source Timing expectations file

- Ticket: `RIOTBOX-770`
- Title: `Add optional local example Source Timing expectations file`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-770/add-optional-local-example-source-timing-expectations-file`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-770-add-optional-local-example-source-timing-expectations-file`
- Linear branch: `feature/riotbox-770-add-optional-local-example-source-timing-expectations-file`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#764 (https://github.com/marang/riotbox/pull/764)`
- Merge commit: `85b637ccf5c893b01ec9853cc98cd6c713f6d5c0`
- Deleted from Linear: `2026-05-11`
- Verification: `just source-timing-example-probe-report-local artifacts/audio_qa/local/source_timing_example_probe_report_expected.md`; `missing-source skip smoke via scripts/source_timing_example_probe_report.py`; `git diff --check`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-769 made the expectation comparison reusable, but the documented local example WAVs still needed an optional expectation profile so local Source Timing probe runs can catch real-example classification drift without committing audio assets.

## What Shipped

- Added a tracked local example Source Timing expectations JSON for the documented example WAV filenames.
- Added a Just recipe that runs the example report against those expectations while keeping missing local WAVs skipped and non-fatal.
- Documented the optional local expectation profile in the Source Timing spec.

## Notes

- The recipe remains local-only because example WAVs are intentionally not committed.
