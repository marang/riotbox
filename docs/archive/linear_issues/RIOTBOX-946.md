# `RIOTBOX-946` Surface generated Feral-grid action cues in P012 proof summary

- Ticket: `RIOTBOX-946`
- Title: `Surface generated Feral-grid action cues in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-946/surface-generated-feral-grid-action-cues-in-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-946-surface-generated-feral-grid-action-cues-in-p012-proof`
- Linear branch: `feature/riotbox-946-surface-generated-feral-grid-action-cues-in-p012-proof`
- Assignee: `Markus`
- Labels: None
- PR: `#939 (https://github.com/marang/riotbox/pull/939)`
- Merge commit: `e9531ff877e4ca920e95818c05b64474e7f56236`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py`; `just p012-all-lane-proof-summary /tmp/riotbox-946-p012-summary.md`; `just p012-all-lane-source-grid-output-proof`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26299154457 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The generated Feral-grid observer/audio index already carried musician-facing Source Timing cue/actionability, but the compact P012 Markdown proof still hid that consequence in raw grid-use fields for generated paths.

## What Shipped

- Added Cue and Action columns to the generated Feral-grid observer/audio table in the compact P012 all-lane proof summary.
- Read generated path cue/actionability from output_path.source_timing so the Markdown proof matches the JSON/TSV contract.
- Tightened the P012 proof summary validator snippets for cautious/manual, user override, risky override, fallback, and locked-grid rows.

## Notes

- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
