# `RIOTBOX-854` Require Source Timing grid_use in observer/audio summary validation

- Ticket: `RIOTBOX-854`
- Title: `Require Source Timing grid_use in observer/audio summary validation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-854/require-source-timing-grid-use-in-observeraudio-summary-validation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-854-observer-source-timing-grid-use-required`
- Linear branch: `feature/riotbox-854-require-source-timing-grid_use-in-observeraudio-summary`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#849 (https://github.com/marang/riotbox/pull/849)`
- Merge commit: `e3a73d268ce1be01fbac73e8822591dff224b9ca`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_source_timing_status_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; git diff --check; GitHub Actions Rust CI #2086 success`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summary validation accepted non-null source_timing objects with missing grid_use, weakening the compact P012 timing-trust contract for edited or external summaries.

## What Shipped

- Required source_timing.grid_use as a present nullable enum field, preserved derived grid-use validation for non-null values, added a missing-grid-use negative fixture, and updated the observer/audio summary contract notes.

## Notes

- None
