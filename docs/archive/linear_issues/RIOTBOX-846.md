# `RIOTBOX-846` Validate observer/audio W-30 loop-closure metric shape

- Ticket: `RIOTBOX-846`
- Title: `Validate observer/audio W-30 loop-closure metric shape`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-846/validate-observeraudio-w-30-loop-closure-metric-shape`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-846-observer-w30-loop-closure-validator`
- Linear branch: `feature/riotbox-846-validate-observeraudio-w-30-loop-closure-metric-shape`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#841 (https://github.com/marang/riotbox/pull/841)`
- Merge commit: `489bcd2169fdcad3a56b5283c49da03a8eb578c6`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_w30_loop_closure_fixtures.py scripts/validate_observer_audio_source_grid_metric_fixtures.py scripts/validate_observer_audio_source_timing_alignment_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI #2062`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summaries documented W-30 loop-closure output evidence under metrics.w30_source_loop_closure, but the stable summary validator did not validate that object, allowing malformed W-30 source-chop proof to pass summary validation.

## What Shipped

- The observer/audio summary JSON validator now validates metrics.w30_source_loop_closure when present, requires booleans for passed and source_contains_selection, requires non-negative numeric RMS/edge/budget fields, adds fixture-generated invalid cases, wires them into just observer-audio-summary-validator-fixtures, and updates observer/audio summary contract docs.

## Notes

- None
