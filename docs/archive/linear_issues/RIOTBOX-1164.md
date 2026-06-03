# `RIOTBOX-1164` P016: Add reserved export.daw_session action boundary guard

- Ticket: `RIOTBOX-1164`
- Title: `P016: Add reserved export.daw_session action boundary guard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1164/p016-add-reserved-exportdaw-session-action-boundary-guard`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1164-p016-add-reserved-exportdaw_session-action-boundary-guard`
- Linear branch: `feature/riotbox-1164-p016-add-reserved-exportdaw_session-action-boundary-guard`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1143 (https://github.com/marang/riotbox/pull/1143)`
- Merge commit: `40e289f805e05dc81c309f91d56360626361ae10`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app reserved_daw_session_export_queue_attempt_is_rejected_without_side_effects -- --nocapture`; `cargo test -p riotbox-core`; `cargo test -p riotbox-app`; `git diff --check`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/specs/action_lexicon_spec.md`; `docs/specs/session_file_spec.md`; `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Reserve DAW-session export as a typed rejected action boundary before the real writer exists.

## What Shipped

- Added ExportDawSession ActionCommand, DAW-session export params and boundary labels, app queue-history rejection guard with destination and receipt intent, tests, and specs; no files, receipts, host checks, observer lifecycle, proof artifacts, or committed Session action are created.

## Notes

- None
