# `RIOTBOX-1022` Require observer downbeat phase evidence in source timing validators

- Ticket: `RIOTBOX-1022`
- Title: `Require observer downbeat phase evidence in source timing validators`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1022/require-observer-downbeat-phase-evidence-in-source-timing-validators`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-28`
- Started: `2026-05-28`
- Finished: `2026-05-28`
- Branch: `feature/riotbox-1022-require-observer-downbeat-phase-evidence-in-source-timing`
- Linear branch: `feature/riotbox-1022-require-observer-downbeat-phase-evidence-in-source-timing`
- Assignee: `Markus`
- Labels: None
- PR: `#1005 (https://github.com/marang/riotbox/pull/1005)`
- Merge commit: `ee84e723b86c730abdd22c4bc1473e27cd16cbcd`
- Deleted from Linear: `2026-05-28`
- Verification: `python3 -m py_compile scripts/validate_user_session_observer_ndjson.py scripts/validate_observer_audio_summary_json.py; just user-session-observer-validator-fixtures; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-json-fixture; just observer-audio-correlate-locked-grid-json-fixture; cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate summarizes_committed_fixture_observer_and_manifest; git diff --check; just ci; GitHub Rust CI #2538 success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md; docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Require observer downbeat phase evidence in source timing validators.

## What Shipped

- Required present observer-side downbeat phase offset, score, score-gap, and alternate-count fields in user-session NDJSON and observer/audio summary validators; refreshed observer fixtures; updated locked-grid alignment expectations to aligned when alternate evidence matches; updated P012 docs and summary contract.

## Notes

- None
