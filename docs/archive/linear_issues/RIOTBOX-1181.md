# `RIOTBOX-1181` P016: Add DAW local-writer export action CLI smoke

- Ticket: `RIOTBOX-1181`
- Title: `P016: Add DAW local-writer export action CLI smoke`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1181/p016-add-daw-local-writer-export-action-cli-smoke`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1181-p016-daw-local-writer-export-action-cli-smoke`
- Linear branch: `feature/riotbox-1181-p016-add-daw-local-writer-export-action-cli-smoke`
- Assignee: `Markus`
- Labels: None
- PR: `#1160 (https://github.com/marang/riotbox/pull/1160)`
- Merge commit: `64bf806165c4a57a24b6ddc49f7de5fe8ce83794`
- Deleted from Linear: `2026-06-04`
- Verification: `cargo fmt; cargo test -p riotbox-app --test daw_session_writer_proof_smoke daw_session_writer_export_execute -- --nocapture; cargo test -p riotbox-app --bin riotbox-app daw_session -- --nocapture; just daw-session-writer-export-execute-smoke; cargo test -p riotbox-app --test daw_session_writer_proof_smoke -- --nocapture; cargo test -p riotbox-app; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The app had an internal export.daw_session local-writer queue/commit path, but the real riotbox-app binary did not expose a compact smoke proving the action boundary, saved Session mutation, and observer lifecycle together.

## What Shipped

- Added --daw-session-writer-export-execute with optional observer output, committed through JamAppState::commit_daw_session_writer_export, saved the mutated Session, added just daw-session-writer-export-execute-smoke, and proved local writer proof files, action log, commit record, writer proof receipt gate, observer lifecycle, and disabled DAW surface blockers.

## Notes

- None
