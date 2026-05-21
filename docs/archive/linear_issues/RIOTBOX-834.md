# `RIOTBOX-834` Compare observer and manifest downbeat offsets in observer/audio alignment

- Ticket: `RIOTBOX-834`
- Title: `Compare observer and manifest downbeat offsets in observer/audio alignment`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-834/compare-observer-and-manifest-downbeat-offsets-in-observeraudio`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-834-downbeat-offset-alignment`
- Linear branch: `feature/riotbox-834-compare-observer-and-manifest-downbeat-offsets-in`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#829 (https://github.com/marang/riotbox/pull/829)`
- Merge commit: `2e8c2c7b9c04367ad658ededf353dc4c5414fdb3`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-app observer_audio_correlate -- --nocapture`; `python3 -m py_compile scripts/validate_observer_audio_summary_json.py`; `python3 -m json.tool crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_downbeat_offset_compatibility.json`; `just observer-audio-summary-validator-fixtures`; `just observer-audio-correlate-generated-feral-grid`; `just observer-audio-correlate-locked-grid-json-fixture`; `just user-session-observer-validator-fixtures`; `just ci`; GitHub Actions Rust CI #2026
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-835`

## Why This Ticket Existed

Observer/audio summaries showed observer and manifest bar-phase evidence after RIOTBOX-833, but alignment did not compare those offsets, so a wrong bar-start phase could hide behind otherwise compatible BPM and grid-use evidence.

## What Shipped

- Added observer/manifest downbeat-offset fields and `downbeat_offset_compatibility` to `source_timing_alignment`, made strict evidence reject numeric offset contradictions, updated Markdown/JSON rendering, validators, fixtures, Justfile gates, and Source Timing spec text.

## Notes

- `RIOTBOX-835` tracks splitting the now-large observer/audio Source Timing alignment tests by evidence family before the next related expansion.
