# `RIOTBOX-1166` P016: Project committed export.daw_session writer lifecycle in observer

- Ticket: `RIOTBOX-1166`
- Title: `P016: Project committed export.daw_session writer lifecycle in observer`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1166/p016-project-committed-exportdaw-session-writer-lifecycle-in-observer`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1166-p016-project-daw-session-writer-lifecycle-in-observer`
- Linear branch: `feature/riotbox-1166-p016-project-committed-exportdaw_session-writer-lifecycle-in`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1145 (https://github.com/marang/riotbox/pull/1145)`
- Merge commit: `d6fad6cd8ded528d39f138ebe16f9192a0e2f18a`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-app export_daw_session_observer -- --nocapture; cargo test -p riotbox-app export_arrangement_observer -- --nocapture; cargo test -p riotbox-app daw_session_writer_export -- --nocapture; cargo test -p riotbox-app reserved_daw_session_export_queue_attempt_is_rejected_without_side_effects -- --nocapture; cargo test -p riotbox-app; cargo test -p riotbox-core; git diff --check; just ci; GitHub rust-ci success`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Project the new bounded export.daw_session local-writer action into observer export lifecycle so operators can see real DAW-session export requests, failures, and completed writer proof from Session truth.

## What Shipped

- Observer export snapshots now include export.daw_session lifecycle records for real queued actions; committed daw_session.local_project_writer_v1 actions attach the matching DAW-session receipt and proof-gate summary via typed receipt_id; reserved rejected attempts produce failed lifecycle without a fake receipt; DAW-session receipt summaries remain read-only when no action exists; specs and focused observer tests were updated.

## Notes

- None
