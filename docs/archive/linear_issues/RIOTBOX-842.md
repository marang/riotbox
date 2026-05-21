# `RIOTBOX-842` Validate Source Timing main alignment status consistency

- Ticket: `RIOTBOX-842`
- Title: `Validate Source Timing main alignment status consistency`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-842/validate-source-timing-main-alignment-status-consistency`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-842-source-timing-main-alignment-validator-consistency`
- Linear branch: `feature/riotbox-842-validate-source-timing-main-alignment-status-consistency`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#837 (https://github.com/marang/riotbox/pull/837)`
- Merge commit: `23fab35ec1bf60e8a2d0b6852978b6086eb3a1d0`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_source_timing_alignment_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI #2050`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The central observer/audio source_timing_alignment block could still be internally contradictory after anchor/groove alignment status was tightened: a summary could claim aligned while carrying issues, or mismatch without a concrete issue.

## What Shipped

- The observer/audio summary JSON validator now requires source_timing_alignment status to agree with its issues list, requires mismatch issues to use the source_timing_alignment. prefix, adds fixture-generated negative cases, and updates the Source Timing spec.

## Notes

- None
