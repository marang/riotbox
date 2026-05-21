# `RIOTBOX-855` Require W-30 loop-closure key in observer/audio summary validation

- Ticket: `RIOTBOX-855`
- Title: `Require W-30 loop-closure key in observer/audio summary validation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-855/require-w-30-loop-closure-key-in-observeraudio-summary-validation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-855-observer-w30-loop-closure-key-required`
- Linear branch: `feature/riotbox-855-require-w-30-loop-closure-key-in-observeraudio-summary`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#850 (https://github.com/marang/riotbox/pull/850)`
- Merge commit: `64837f4cd8ebe763b899dea013ae7ff7130253c9`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_w30_loop_closure_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; git diff --check; GitHub Actions Rust CI #2089 success`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summary validation accepted missing metrics.w30_source_loop_closure keys even though the stable metric contract requires the key as null or object evidence.

## What Shipped

- Required w30_source_loop_closure as a present nullable metric key, preserved existing object validation, added missing-key fixture coverage, updated older summary fixtures with null, and updated the observer/audio summary contract notes.

## Notes

- None
