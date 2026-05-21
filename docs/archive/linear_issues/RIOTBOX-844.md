# `RIOTBOX-844` Reject negative Source Timing alignment BPM numbers

- Ticket: `RIOTBOX-844`
- Title: `Reject negative Source Timing alignment BPM numbers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-844/reject-negative-source-timing-alignment-bpm-numbers`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-844-source-timing-alignment-nonnegative-bpm`
- Linear branch: `feature/riotbox-844-reject-negative-source-timing-alignment-bpm-numbers`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#839 (https://github.com/marang/riotbox/pull/839)`
- Merge commit: `2c4419ce729c5bc5dd7b99f3b48aed20e77aab17`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_source_timing_alignment_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI #2056`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

The observer/audio summary contract defines source_timing_alignment.bpm_delta as an absolute BPM difference and bpm_tolerance as the comparison tolerance, but the validator still accepted negative values after RIOTBOX-843 began deriving mismatch issues from those numbers.

## What Shipped

- The observer/audio summary JSON validator now rejects negative source_timing_alignment.bpm_delta and bpm_tolerance values, adds fixture-generated invalid cases for both, and documents the non-negative contract.

## Notes

- None
