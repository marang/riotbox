# `RIOTBOX-1069` Specify next export scope fields before stems or DAW export

- Ticket: `RIOTBOX-1069`
- Title: `Specify next export scope fields before stems or DAW export`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1069/specify-next-export-scope-fields-before-stems-or-daw-export`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1069-export-scope-contract`
- Linear branch: `feature/riotbox-1069-specify-next-export-scope-fields-before-stems-or-daw-export`
- Assignee: `Markus`
- Labels: None
- PR: `#1045 (https://github.com/marang/riotbox/pull/1045)`
- Merge commit: `2ebfd4a4db276508a299062740f892162bf87c90`
- Deleted from Linear: `2026-05-31`
- Verification: `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1045`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/reviews/p016_next_export_scope_contract_2026-05-31.md`
- Follow-ups: `RIOTBOX-1071, RIOTBOX-1072`

## Why This Ticket Existed

The broader P016 export roadmap needs typed contracts before stem package, live recording, DAW session, or host-audio claims can safely be implemented.

## What Shipped

- Reserved wider export command names without implementing them.
- Specified required Action Lexicon fields for artifact sets, timing, receipts, observer lifecycle, and audio-QA gates.
- Specified additional Session/Core receipt fields required before wider export scopes.
- Specified stronger Audio QA gates for stem package, live recording, and DAW session claims.
- Added P016 next export scope review and created follow-up tickets RIOTBOX-1071 and RIOTBOX-1072.

## Notes

- No stem, DAW, live recording, or host-audio export implementation shipped in this ticket.
