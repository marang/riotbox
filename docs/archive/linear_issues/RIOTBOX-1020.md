# `RIOTBOX-1020` Require observer source timing actionability in control path validators

- Ticket: `RIOTBOX-1020`
- Title: `Require observer source timing actionability in control path validators`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1020/require-observer-source-timing-actionability-in-control-path`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-28`
- Started: `2026-05-28`
- Finished: `2026-05-28`
- Branch: `feature/riotbox-1020-require-observer-source-timing-actionability-in-control-path`
- Linear branch: `feature/riotbox-1020-require-observer-source-timing-actionability-in-control-path`
- Assignee: `Markus`
- Labels: None
- PR: `#1003 (https://github.com/marang/riotbox/pull/1003)`
- Merge commit: `72fcb37884c17c14531eb4ad144485109be238bf`
- Deleted from Linear: `2026-05-28`
- Verification: `python3 -m py_compile scripts/validate_user_session_observer_ndjson.py scripts/validate_observer_audio_summary_json.py; just user-session-observer-validator-fixtures; just observer-audio-summary-validator-fixtures; cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate observer_source_timing; cargo test -p riotbox-app --bin observer_audio_correlate source_timing_alignment; just observer-audio-correlate-json-fixture; just observer-audio-correlate-locked-grid-json-fixture; git diff --check; just ci; GitHub Rust CI #2532 success`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Require observer source timing actionability in control path validators.

## What Shipped

- Required policy-derived source_timing.actionability in user-session observer NDJSON, required control_path.observer_source_timing.actionability in observer/audio summary JSON, tightened Rust strict observer correlation, refreshed fixtures, added missing-actionability negative gates, and updated the summary JSON contract.

## Notes

- None
