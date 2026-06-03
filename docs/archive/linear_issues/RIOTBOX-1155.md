# `RIOTBOX-1155` P016: Add DAW host-import proof evidence apply CLI

- Ticket: `RIOTBOX-1155`
- Title: `P016: Add DAW host-import proof evidence apply CLI`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1155/p016-add-daw-host-import-proof-evidence-apply-cli`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1155-p016-add-daw-host-import-proof-evidence-apply-cli`
- Linear branch: `feature/riotbox-1155-p016-add-daw-host-import-proof-evidence-apply-cli`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1134 (https://github.com/marang/riotbox/pull/1134)`
- Merge commit: `59e0442d959e20f17ba1e2dd83c5b67312aaed44`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; cargo test -p riotbox-app; just daw-session-host-import-proof-apply-smoke; cargo clippy --all-targets --all-features -- -D warnings; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `Implement the actual DAW host runner/import proof producer and add DAW-session audible-output proof before enabling export.daw_session.`

## Why This Ticket Existed

Bridge external DAW host-import proof into typed Session receipt QA evidence without creating a shadow export state.

## What Shipped

- Added riotbox.daw_session_host_import_proof report reader, guarded --daw-session-host-import-proof-apply CLI, receipt QA gate attachment, binary smoke target, and P016 docs.

## Notes

- The path writes only the Session file and only updates daw_session_host_import_proof. It launches no host, writes no DAW files, emits no observer lifecycle, claims no audible output, and keeps export.daw_session disabled.
