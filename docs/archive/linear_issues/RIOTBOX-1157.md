# `RIOTBOX-1157` P016: Add DAW session audible-output proof apply CLI

- Ticket: `RIOTBOX-1157`
- Title: `P016: Add DAW session audible-output proof apply CLI`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1157/p016-add-daw-session-audible-output-proof-apply-cli`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1157-p016-add-daw-session-audible-output-proof-apply-cli`
- Linear branch: `feature/riotbox-1157-p016-add-daw-session-audible-output-proof-apply-cli`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1136 (https://github.com/marang/riotbox/pull/1136)`
- Merge commit: `48e031bf0174c87ddd7a9197502d187767911058`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-app daw_session_audible_output_proof -- --nocapture; cargo test -p riotbox-app parse_args_builds_daw_session -- --nocapture; just daw-session-audible-output-proof-apply-smoke; cargo test -p riotbox-core; cargo test -p riotbox-app; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1158 defines the first DAW session writer action boundary before export.daw_session can become musician-runnable.`

## Why This Ticket Existed

Bridge external DAW audible-output evidence into typed Session receipt truth without overclaiming DAW playback/export readiness.

## What Shipped

- Added riotbox.daw_session_audible_output_proof report parsing, --daw-session-audible-output-proof-apply CLI, Session-only receipt gate mutation, real-binary smoke proof, DAW-session arg helper extraction, and spec updates.

## Notes

- Passed audible-output proof removes only audible_output_proof_missing. developer_proof_only, daw_writer_missing, and any missing host-import/package blockers remain; no DAW writer, host launch, live audio capture, observer lifecycle, or runnable export.daw_session was added.
