# `RIOTBOX-953` Show generated phrase evidence in P012 proof summary

- Ticket: `RIOTBOX-953`
- Title: `Show generated phrase evidence in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-953/show-generated-phrase-evidence-in-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-953-show-generated-phrase-evidence-in-p012-proof-summary`
- Linear branch: `feature/riotbox-953-show-generated-phrase-evidence-in-p012-proof-summary`
- Assignee: `Markus`
- Labels: None
- PR: `#946 (https://github.com/marang/riotbox/pull/946)`
- Merge commit: `f2d7ab958e050b275ccdb4ece566d87ab11a2fef`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py`; `just p012-all-lane-proof-summary /tmp/riotbox-953-p012-summary.md`; `scripts/run_compact.sh /tmp/riotbox-953-p012-proof.log just p012-all-lane-source-grid-output-proof`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-953-just-ci.log just ci`; `git diff --check main...HEAD`; `just p012-all-lane-proof-summary /tmp/riotbox-953-p012-summary-rebased.md`; `GitHub Actions Rust CI run 26301498805 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-952 found that the compact P012 Markdown proof still hid generated-path phrase evidence after the TSV gained phrase count/bar fields.

## What Shipped

- Added Phrase count and Phrase bars to the Generated Feral-Grid Observer/Audio Paths table.
- Read generated phrase evidence from output_path.source_timing.primary_phrase_count and primary_phrase_bar_count.
- Pinned validator snippets for cautious/manual, user override, risky override, fallback, and locked-grid generated rows.

## Notes

- Display/validator-only slice; no analyzer, ActionCommand, queue, Session/replay, JamAppState, realtime audio, or render behavior changed.
