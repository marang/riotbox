# `RIOTBOX-1266` MC-202 source-backed candidate families

- Ticket: `RIOTBOX-1266`
- Title: `MC-202 source-backed candidate families`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1266/mc-202-source-backed-candidate-families`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-14`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1266-mc202-source-backed-candidate-families`
- Linear branch: `feature/riotbox-1266-mc-202-source-backed-candidate-families`
- Assignee: `Markus`
- Labels: None
- PR: `#1241 (https://github.com/marang/riotbox/pull/1241)`
- Merge commit: `9f22f608aa9701a8e6b6fe96cff24ca50f116e6c`
- Deleted from Linear: `2026-06-15`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

MC-202 source phrase planning still looked like one opaque source-aware template. Riotbox needed replayable candidate families and rejection metadata so source-derived bass/answer claims can be audited instead of inferred from logs.

## What Shipped

- Added source-backed MC-202 candidate families, persisted selected family/count/rejection/provenance in Session, kept fallback/control candidates rejected from source-derived proof, and added tests for candidate metadata, cross-source family changes, rendered output delta, and measured-audio removal.

## Notes

- None
