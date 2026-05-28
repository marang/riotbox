# `RIOTBOX-1019` Require output source timing actionability in observer audio summaries

- Ticket: `RIOTBOX-1019`
- Title: `Require output source timing actionability in observer audio summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1019/require-output-source-timing-actionability-in-observer-audio-summaries`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-28`
- Started: `2026-05-28`
- Finished: `2026-05-28`
- Branch: `feature/riotbox-1019-require-output-source-timing-actionability-in-observer-audio`
- Linear branch: `feature/riotbox-1019-require-output-source-timing-actionability-in-observer-audio`
- Assignee: `Markus`
- Labels: None
- PR: `#1002 (https://github.com/marang/riotbox/pull/1002)`
- Merge commit: `4f7dbbc201a2ca8ff3e2eada5cf6143268774996`
- Deleted from Linear: `2026-05-28`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py; missing-actionability repro fails; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-json-fixture; just observer-audio-correlate-locked-grid-json-fixture; git diff --check; just ci; GitHub Rust CI 26585405075 success`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `none`

## Why This Ticket Existed

Observer/audio summary JSON validation still accepted output_path.source_timing without the musician-facing actionability field, even though current P012 generated manifests and proof surfaces require cue/actionability evidence.

## What Shipped

- Required output_path.source_timing.actionability when source timing evidence is non-null, added a missing-actionability negative validator gate, refreshed static source-timing summary fixtures, and updated the JSON contract note.

## Notes

- No new summary schema version or runtime timing authority was introduced; the validator now enforces the current P012 field contract.
