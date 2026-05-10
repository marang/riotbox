# `RIOTBOX-19` Implement decoded-source analysis baseline behind the ingest path

- Ticket: `RIOTBOX-19`
- Title: `Implement decoded-source analysis baseline behind the ingest path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-19/implement-decoded-source-analysis-baseline-behind-the-ingest-path`
- Project: `P003 | Analysis Vertical Slice`
- Milestone: `Analysis Vertical Slice`
- Status: `Done`
- Created: `2026-04-12`
- Finished: `2026-04-12`
- Branch: `lemonsterizoone/riotbox-19-implement-decoded-source-analysis-baseline-behind-the-ingest`
- PR: `#13`
- Merge commit: `23eca37`
- Follow-ups: `RIOTBOX-20`

## Why This Ticket Existed

The first ingest seam needed a more real decoded-source baseline instead of file heuristics.

## What Shipped

- Added WAV decoding and bounded decoded-source facts inside the Python sidecar.

## Notes

- This kept the same ingest architecture while making analysis materially more real.
