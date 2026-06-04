# `RIOTBOX-1179` P016: Add live-recording reserved action lifecycle smoke

- Ticket: `RIOTBOX-1179`
- Title: `P016: Add live-recording reserved action lifecycle smoke`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1179/p016-add-live-recording-reserved-action-lifecycle-smoke`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1179-p016-live-recording-reserved-action-lifecycle-smoke`
- Linear branch: `feature/riotbox-1179-p016-add-live-recording-reserved-action-lifecycle-smoke`
- Assignee: `Markus`
- Labels: None
- PR: `#1158 (https://github.com/marang/riotbox/pull/1158)`
- Merge commit: `2b7b75a769fbfa880b150afa2933e090c1dd83c9`
- Deleted from Linear: `2026-06-04`
- Verification: `just live-recording-reserved-action-lifecycle-smoke`; `cargo test -p riotbox-app`; `git diff --check`; `just ci`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Live-recording export is still intentionally reserved, but P016 needed an executable proof that a user attempt produces failed lifecycle evidence from the typed action boundary without creating receipt evidence or destination files. This prevents read-only receipt projection from being confused with successful capture.

## What Shipped

- Added just live-recording-reserved-action-lifecycle-smoke for the rejected export.live_recording observer path.
- Tightened the rejected lifecycle assertions to prove export presence, rejected status, null receipt, no destination, and matching failure/result reason.
- Documented the smoke in Action Lexicon, Audio QA, and Session specs.

## Notes

- PR #1158 merged after GitHub rust-ci passed.
