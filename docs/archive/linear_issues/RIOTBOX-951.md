# `RIOTBOX-951` Add phrase evidence to generated Feral-grid summary index

- Ticket: `RIOTBOX-951`
- Title: `Add phrase evidence to generated Feral-grid summary index`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-951/add-phrase-evidence-to-generated-feral-grid-summary-index`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-951-add-phrase-evidence-to-generated-feral-grid-summary-index`
- Linear branch: `feature/riotbox-951-add-phrase-evidence-to-generated-feral-grid-summary-index`
- Assignee: `Markus`
- Labels: None
- PR: `#944 (https://github.com/marang/riotbox/pull/944)`
- Merge commit: `e322c579f8f9a02c30c77b7e132aff57cbc45089`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-generated-feral-grid`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26300906034 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Generated Feral-grid observer/audio JSON summaries preserved phrase evidence, but the compact summary.tsv index hid primary_phrase_count and primary_phrase_bar_count, forcing reviewers to open JSON to distinguish short-loop, unavailable, and locked phrase evidence.

## What Shipped

- Added phrase_count and phrase_bars columns to the generated Feral-grid observer/audio summary.tsv.
- Populated the columns from output_path.source_timing.primary_phrase_count and primary_phrase_bar_count.
- Pinned TSV greps for cautious/manual, fallback, and locked-grid rows, including the current locked-grid evidence of 2 phrases across 9 bars.

## Notes

- Initial local gate caught an incorrect locked-grid expectation and was corrected before passing CI.
- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
