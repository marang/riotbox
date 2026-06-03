# `RIOTBOX-1171` P016: Define first live-recording export receipt boundary

- Ticket: `RIOTBOX-1171`
- Title: `P016: Define first live-recording export receipt boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1171/p016-define-first-live-recording-export-receipt-boundary`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1171-p016-live-recording-export-boundary`
- Linear branch: `feature/riotbox-1171-p016-define-first-live-recording-export-receipt-boundary`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1150 (https://github.com/marang/riotbox/pull/1150)`
- Merge commit: `b55631bc86d37ad9c0afa021dda90783956afc23`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; cargo test -p riotbox-core action::tests; cargo test -p riotbox-core session::export_types_tests; cargo test -p riotbox-core; cargo test -p riotbox-app; just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1172: Extract export action and receipt contract tests from oversized Core modules.`

## Why This Ticket Existed

P016 needs a typed live-recording export action and receipt identity before any live capture or writer side effect, so future recording work does not become hidden JamAppState or observer-only truth.

## What Shipped

- Reserved export.live_recording, LiveRecordingExport params, live_recording receipt scope, live_recording.receipt_contract_v1 boundary, live_recording_capture role/artifact helper, observer export-action recognition, replay family labeling, and specs for the no-writer/no-capture boundary.

## Notes

- No live input capture, WAV writer, queue/commit helper, Session receipt mutation, observer completion, TUI affordance, Ghost affordance, or replay write-back was added.
