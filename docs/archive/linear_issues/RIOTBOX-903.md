# `RIOTBOX-903` Carry Source Timing phrase counts into observer/audio summaries

- Ticket: `RIOTBOX-903`
- Title: `Carry Source Timing phrase counts into observer/audio summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-903/carry-source-timing-phrase-counts-into-observeraudio-summaries`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-903-observer-audio-phrase-counts`
- Linear branch: `feature/riotbox-903-carry-source-timing-phrase-counts-into-observeraudio`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#896 (https://github.com/marang/riotbox/pull/896)`
- Merge commit: `b8a6ba5eb5594019ba56413011eff33b958dfa4c`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; python3 -m py_compile scripts/validate_observer_audio_summary_json.py; cargo test -p riotbox-app --bin observer_audio_correlate; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-json-fixture; just observer-audio-correlate-locked-grid-json-fixture; manual JSON spot checks; git diff --check; just ci; GitHub Rust CI #2231`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md; docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summaries dropped manifest-side Source Timing phrase-count evidence, leaving downstream QA unable to distinguish no phrase grid, short-loop material, and stable preliminary phrase material from the summary alone.

## What Shipped

- Added primary_phrase_count and primary_phrase_bar_count to observer/audio source_timing model, JSON, Markdown phrase line, validator fixtures, and contracts.

## Notes

- None
