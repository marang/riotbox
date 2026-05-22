# `RIOTBOX-948` Show downbeat ambiguity compatibility in P012 proof summary

- Ticket: `RIOTBOX-948`
- Title: `Show downbeat ambiguity compatibility in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-948/show-downbeat-ambiguity-compatibility-in-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-948-show-downbeat-ambiguity-compatibility-in-p012-proof-summary`
- Linear branch: `feature/riotbox-948-show-downbeat-ambiguity-compatibility-in-p012-proof-summary`
- Assignee: `Markus`
- Labels: None
- PR: `#941 (https://github.com/marang/riotbox/pull/941)`
- Merge commit: `e2a977ce40c50efcc1d9f7aec99d1c3847d67732`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py`; `just p012-all-lane-proof-summary /tmp/riotbox-948-p012-summary.md`; `just p012-all-lane-source-grid-output-proof`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26299783680 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The compact P012 all-lane Markdown proof summary showed generated-path downbeat-offset compatibility but hid the separate ambiguity-compatibility verdict, so reviewers could not distinguish partial bar-phase ambiguity evidence from locked-grid agreement without opening JSON summaries.

## What Shipped

- Added a Downbeat ambiguity column to the Generated Feral-Grid Observer/Audio Paths table in the compact P012 proof summary.
- Read the value from output_path.source_timing_alignment.downbeat_ambiguity_compatibility.
- Tightened proof-summary validator snippets for cautious/manual, user override, risky override, fallback, and locked-grid rows.

## Notes

- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
