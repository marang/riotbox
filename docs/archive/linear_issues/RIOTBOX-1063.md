# `RIOTBOX-1063` Draft P016 export action boundary before file-writing workflow

- Ticket: `RIOTBOX-1063`
- Title: `Draft P016 export action boundary before file-writing workflow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1063/draft-p016-export-action-boundary-before-file-writing-workflow`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1063-export-action-boundary`
- Linear branch: `feature/riotbox-1063-draft-p016-export-action-boundary-before-file-writing`
- Assignee: `Markus`
- Labels: None
- PR: `#1039 (https://github.com/marang/riotbox/pull/1039)`
- Merge commit: `8960cdd5434747b81b97e1361b9884b5cd376dde`
- Deleted from Linear: `2026-05-31`
- Verification: `git diff --check: pass`; `targeted rg for export action, receipt, and unsupported-scope language: pass`; `just ci: pass`; `GitHub rust-ci on PR #1039: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Before Riotbox adds a user-triggered export command, the ActionCommand, Session/replay, observer, and QA consequences needed to be explicit.

## What Shipped

- Added docs/reviews/p016_export_action_boundary_2026-05-31.md.
- Reserved the future export.product_mix action boundary in docs/specs/action_lexicon_spec.md.
- Added future export receipt requirements to docs/specs/session_file_spec.md.
- Updated docs/specs/audio_qa_workflow_spec.md with the export-action QA boundary.
- Linked the review from docs/README.md.
- Created follow-up Linear tickets RIOTBOX-1064, RIOTBOX-1065, and RIOTBOX-1066.

## Notes

- None
