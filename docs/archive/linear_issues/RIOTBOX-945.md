# `RIOTBOX-945` Write compact generated Feral-grid source-timing summary index

- Ticket: `RIOTBOX-945`
- Title: `Write compact generated Feral-grid source-timing summary index`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-945/write-compact-generated-feral-grid-source-timing-summary-index`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-945-generated-feral-grid-summary-index`
- Linear branch: `feature/riotbox-945-write-compact-generated-feral-grid-source-timing-summary`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#938 (https://github.com/marang/riotbox/pull/938)`
- Merge commit: `b092c603fab9cc856375b83d4f9f783ba806a537`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-generated-feral-grid; just ci; GitHub Actions Rust CI run 26298645137 passed`
- Docs touched: `none`
- Follow-ups: `Continue P012 from the source-timing roadmap; generated Feral-grid proof now leaves a compact TSV summary artifact.`

## Why This Ticket Existed

Generated Feral-grid QA copied per-case JSON summaries but did not leave a single compact artifact for reviewers to inspect source-timing decisions and cues without scraping logs.

## What Shipped

- Added artifacts/audio_qa/local/generated-feral-grid-observer-audio/summary.tsv with case, grid BPM source, decision reason, cue, actionability, grid use, alignment, and issue count; added assertions for expected cautious, fallback, and locked rows.

## Notes

- QA artifact usability only; no runtime behavior changed.
