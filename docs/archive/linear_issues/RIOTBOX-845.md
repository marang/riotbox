# `RIOTBOX-845` Validate observer/audio source-grid metric numeric ranges

- Ticket: `RIOTBOX-845`
- Title: `Validate observer/audio source-grid metric numeric ranges`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-845/validate-observeraudio-source-grid-metric-numeric-ranges`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-845-observer-source-grid-metric-ranges`
- Linear branch: `feature/riotbox-845-validate-observeraudio-source-grid-metric-numeric-ranges`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#840 (https://github.com/marang/riotbox/pull/840)`
- Merge commit: `158dad0ffa63f3f5727a71a32ca0999c53ed9430`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_source_grid_metric_fixtures.py scripts/validate_observer_audio_source_timing_alignment_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI #2059`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summaries expose pack-level and lane-specific source-grid metrics, but the JSON validator only checked that their fields were numeric. Impossible values such as hit_ratio > 1 or negative offsets could pass even though the listening-manifest validator already enforces range semantics.

## What Shipped

- The observer/audio summary JSON validator now requires source-grid metric hit_ratio in [0, 1], rejects negative max_peak_offset_ms and max_allowed_peak_offset_ms, adds fixture-generated invalid cases for pack-level/TR-909/MC-202/W-30 metrics, wires them into just observer-audio-summary-validator-fixtures, and updates the observer/audio summary contract doc.

## Notes

- None
