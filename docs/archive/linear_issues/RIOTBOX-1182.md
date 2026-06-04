# `RIOTBOX-1182` P016: Add DAW host-import proof export-action CLI smoke

- Ticket: `RIOTBOX-1182`
- Title: `P016: Add DAW host-import proof export-action CLI smoke`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1182/p016-add-daw-host-import-proof-export-action-cli-smoke`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1182-p016-daw-host-import-proof-export-action-cli-smoke`
- Linear branch: `feature/riotbox-1182-p016-add-daw-host-import-proof-export-action-cli-smoke`
- Assignee: `Markus`
- Labels: None
- PR: `#1161 (https://github.com/marang/riotbox/pull/1161)`
- Merge commit: `8a36183c3a9801ee9b9096ee7ce34cc0d13016fe`
- Deleted from Linear: `2026-06-04`
- Verification: `cargo fmt; cargo test -p riotbox-app --test daw_session_host_import_proof_apply_smoke daw_session_host_import_proof_export_execute -- --nocapture; cargo test -p riotbox-app --bin riotbox-app daw_session -- --nocapture; just daw-session-host-import-proof-export-execute-smoke; cargo test -p riotbox-app --test daw_session_host_import_proof_apply_smoke -- --nocapture; cargo test -p riotbox-app; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Host-import proof had direct receipt apply coverage and app-level action tests, but no real riotbox-app binary path proving the export.daw_session host_import_proof_v1 action boundary end to end.

## What Shipped

- Added --daw-session-host-import-proof-export-execute with optional observer output, committed through JamAppState::commit_daw_session_host_import_proof_export, saved the mutated Session, added just daw-session-host-import-proof-export-execute-smoke, and proved action log, commit record, host-import receipt gate, observer lifecycle, and remaining developer_proof_only/audible_output blockers without launching a host or writing DAW files.

## Notes

- None
