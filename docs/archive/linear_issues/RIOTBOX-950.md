# `RIOTBOX-950` Show anchor and groove alignment in P012 proof summary

- Ticket: `RIOTBOX-950`
- Title: `Show anchor and groove alignment in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-950/show-anchor-and-groove-alignment-in-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-950-show-anchor-and-groove-alignment-in-p012-proof-summary`
- Linear branch: `feature/riotbox-950-show-anchor-and-groove-alignment-in-p012-proof-summary`
- Assignee: `Markus`
- Labels: None
- PR: `#943 (https://github.com/marang/riotbox/pull/943)`
- Merge commit: `937801343545ef004be0f2f7695f7448ed453565`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py`; `just p012-all-lane-proof-summary /tmp/riotbox-950-p012-summary.md`; `just p012-all-lane-source-grid-output-proof`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26300403308 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The compact P012 all-lane Markdown proof summary exposed generated-path cue/action and downbeat ambiguity, but hid anchor and groove alignment status, so reviewers could not tell whether timing-anchor and microtiming evidence were partial or aligned without opening JSON summaries.

## What Shipped

- Added Anchor alignment and Groove alignment columns to the Generated Feral-Grid Observer/Audio Paths table in the compact P012 proof summary.
- Read both columns from output_path.source_timing_anchor_alignment.status and output_path.source_timing_groove_alignment.status.
- Tightened proof-summary validator snippets for cautious/manual, user override, risky override, fallback, and locked-grid rows.

## Notes

- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
