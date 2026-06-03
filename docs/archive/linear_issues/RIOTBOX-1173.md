# `RIOTBOX-1173` P016: Add reserved live-recording export queue guard

- Ticket: `RIOTBOX-1173`
- Title: `P016: Add reserved live-recording export queue guard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1173/p016-add-reserved-live-recording-export-queue-guard`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1173-p016-live-recording-export-queue-guard`
- Linear branch: `feature/riotbox-1173-p016-add-reserved-live-recording-export-queue-guard`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1152 (https://github.com/marang/riotbox/pull/1152)`
- Merge commit: `9883fd6a65d629cc7039011f5910b2956a63d2f9`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; focused live-recording app/observer tests; cargo test -p riotbox-app; cargo test -p riotbox-core; cargo check -p riotbox-app; cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/session_file_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 had a typed live-recording export contract, but app code still had no bounded queue surface for attempted musician actions.

## What Shipped

- Added a reserved export.live_recording queue guard that records a rejected typed LiveRecordingExport action, emits observer requested/started/failed lifecycle evidence, and writes no files or Session receipts. Updated Action Lexicon, Session file, and Audio QA specs to keep the boundary explicit.

## Notes

- None
