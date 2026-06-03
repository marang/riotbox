# `RIOTBOX-1168` P016: Commit DAW audible-output proof through export.daw_session action

- Ticket: `RIOTBOX-1168`
- Title: `P016: Commit DAW audible-output proof through export.daw_session action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1168/p016-commit-daw-audible-output-proof-through-exportdaw-session-action`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1168-p016-commit-daw-audible-output-proof-through-exportdaw-session`
- Linear branch: `feature/riotbox-1168-p016-commit-daw-audible-output-proof-through`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1147 (https://github.com/marang/riotbox/pull/1147)`
- Merge commit: `8647f130`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-app daw_session_audible_output_proof -- --nocapture; cargo test -p riotbox-app export_daw_session_observer -- --nocapture; cargo test -p riotbox-app daw_session_host_import_proof -- --nocapture; cargo test -p riotbox-core daw_session_export_action_contract_roundtrips_as_reserved_scope -- --nocapture; cargo test -p riotbox-app; cargo test -p riotbox-core; git diff --check; scripts/run_compact.sh /tmp/riotbox-1168-just-ci-rerun.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Move DAW audible-output proof from a standalone receipt mutation into the export.daw_session queue, commit, Session, and observer spine while preserving the final developer_proof_only release guard.

## What Shipped

- Added audible_output_proof_v1 boundary; queued and committed existing local audible-output proof after writer and host-import proof; mutated the matching queued DAW receipt; recorded action/commit evidence; projected observer lifecycle; kept developer_proof_only visible; added latest-receipt drift and stale-receipt regressions.

## Notes

- This still does not launch a DAW host, write DAW files, record live audio, or make DAW export musician-runnable.
