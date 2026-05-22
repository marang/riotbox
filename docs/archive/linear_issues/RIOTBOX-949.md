# `RIOTBOX-949` Add anchor and groove alignment to generated Feral-grid summary index

- Ticket: `RIOTBOX-949`
- Title: `Add anchor and groove alignment to generated Feral-grid summary index`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-949/add-anchor-and-groove-alignment-to-generated-feral-grid-summary-index`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-949-add-anchor-and-groove-alignment-to-generated-feral-grid`
- Linear branch: `feature/riotbox-949-add-anchor-and-groove-alignment-to-generated-feral-grid`
- Assignee: `Markus`
- Labels: None
- PR: `#942 (https://github.com/marang/riotbox/pull/942)`
- Merge commit: `22fd478ff4aa7a0b6d3739f4b4d54678ab89c541`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-generated-feral-grid`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26300070306 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Generated Feral-grid observer/audio JSON summaries already carried anchor and groove alignment status, but the compact summary.tsv index hid those status fields, forcing reviewers to open JSON to see whether evidence was partial or aligned.

## What Shipped

- Added anchor_alignment and groove_alignment columns to the generated Feral-grid observer/audio summary.tsv.
- Populated the columns from output_path.source_timing_anchor_alignment.status and output_path.source_timing_groove_alignment.status.
- Pinned TSV greps for cautious/manual, fallback, and locked-grid rows so partial vs aligned anchor/groove evidence remains visible.

## Notes

- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
