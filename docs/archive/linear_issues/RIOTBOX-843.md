# `RIOTBOX-843` Validate Source Timing alignment BPM-delta issue derivation

- Ticket: `RIOTBOX-843`
- Title: `Validate Source Timing alignment BPM-delta issue derivation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-843/validate-source-timing-alignment-bpm-delta-issue-derivation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-843-source-timing-alignment-bpm-delta-validator`
- Linear branch: `feature/riotbox-843-validate-source-timing-alignment-bpm-delta-issue-derivation`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#838 (https://github.com/marang/riotbox/pull/838)`
- Merge commit: `26a6d26c68cd938af0dbf4f81e72e8af9848a400`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_source_timing_alignment_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI #2053`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

After RIOTBOX-842 made source_timing_alignment status coherent with issues, the standalone observer/audio summary validator still did not derive BPM mismatch issues from bpm_delta and bpm_tolerance, so stale or missing BPM issues could pass validation.

## What Shipped

- The observer/audio summary JSON validator now requires out-of-tolerance source_timing_alignment BPM deltas to carry a source_timing_alignment.bpm_delta issue, rejects BPM issues without a numeric mismatch, adds fixture-generated negative cases, and updates the observer/audio summary contract doc.

## Notes

- None
