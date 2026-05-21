# `RIOTBOX-833` Propagate observer downbeat offset into observer/audio summaries

- Ticket: `RIOTBOX-833`
- Title: `Propagate observer downbeat offset into observer/audio summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-833/propagate-observer-downbeat-offset-into-observeraudio-summaries`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-833-observer-audio-downbeat-offset`
- Linear branch: `feature/riotbox-833-propagate-observer-downbeat-offset-into-observeraudio`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#828 (https://github.com/marang/riotbox/pull/828)`
- Merge commit: `2c047664a14c06d5f50a73d396823b90032d0965`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-app observer_audio_correlate -- --nocapture`; `python3 -m py_compile scripts/validate_observer_audio_summary_json.py`; `just observer-audio-summary-validator-fixtures`; `just observer-audio-correlate-generated-feral-grid`; `just ci`; GitHub Actions Rust CI #2023
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio QA summaries needed to carry the observer-side primary downbeat offset from RIOTBOX-832 so reviewers can compare control-path bar-phase evidence with manifest-side Source Timing evidence in one artifact.

## What Shipped

- Added `control_path.observer_source_timing.primary_downbeat_offset_beats` to observer/audio JSON, rendered observer offset in Markdown summaries, updated validators/fixtures, and documented the Source Timing correlation contract.

## Notes

- None
