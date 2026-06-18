# `RIOTBOX-1287` Extract edge and non-dense professional JSON gates from Justfile

- Ticket: `RIOTBOX-1287`
- Title: `Extract edge and non-dense professional JSON gates from Justfile`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1287/extract-edge-and-non-dense-professional-json-gates-from-justfile`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1287-edge-non-dense-json-validators`
- Linear branch: `feature/riotbox-1287-extract-edge-and-non-dense-professional-json-gates-from`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1262 (https://github.com/marang/riotbox/pull/1262)`
- Merge commit: `b5edc230e1007db14d5d68500df431ab66a45dea`
- Deleted from Linear: `2026-06-18`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Edge-source and non-dense professional QA gates had grown into long inline Justfile jq contracts, making them hard to review and easy to weaken while P023 sound-quality diagnostics evolve.

## What Shipped

- Moved edge-source and non-dense report checks into named Python validator modes with artifact requirements and mutation fixtures, shortened the Justfile smokes, and documented that these scripted diagnostics remain diagnostic-only rather than product-quality proof.

## Notes

- None
