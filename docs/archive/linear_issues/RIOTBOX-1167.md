# `RIOTBOX-1167` P016: Commit DAW host-import proof through export.daw_session action

- Ticket: `RIOTBOX-1167`
- Title: `P016: Commit DAW host-import proof through export.daw_session action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1167/p016-commit-daw-host-import-proof-through-exportdaw-session-action`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1167-p016-commit-daw-host-import-proof-through-exportdaw-session`
- Linear branch: `feature/riotbox-1167-p016-commit-daw-host-import-proof-through-exportdaw_session`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1146 (https://github.com/marang/riotbox/pull/1146)`
- Merge commit: `84d656aa`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-app daw_session_host_import_proof -- --nocapture; cargo test -p riotbox-app export_daw_session_observer -- --nocapture; cargo test -p riotbox-app daw_session_writer_export -- --nocapture; cargo test -p riotbox-core daw_session_export_action_contract_roundtrips_as_reserved_scope -- --nocapture; cargo test -p riotbox-app; cargo test -p riotbox-core; git diff --check; scripts/run_compact.sh /tmp/riotbox-1167-just-ci-rerun.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Move DAW host-import proof from a standalone receipt mutation into the export.daw_session queue, commit, Session, and observer spine before wider DAW export enablement.

## What Shipped

- Added host_import_proof_v1 boundary; queued and committed existing local host-import proof against the matching queued DAW receipt; recorded action/commit evidence; projected observer lifecycle; kept developer_proof_only and audible_output_proof_missing visible; added receipt-drift and stale-receipt regressions.

## Notes

- This still does not launch a DAW host, write DAW files, capture audio, or prove audible output.
