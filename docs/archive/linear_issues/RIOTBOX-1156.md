# `RIOTBOX-1156` P016: Add DAW session audible-output proof gate boundary

- Ticket: `RIOTBOX-1156`
- Title: `P016: Add DAW session audible-output proof gate boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1156/p016-add-daw-session-audible-output-proof-gate-boundary`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1156-p016-add-daw-session-audible-output-proof-gate-boundary`
- Linear branch: `feature/riotbox-1156-p016-add-daw-session-audible-output-proof-gate-boundary`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1135 (https://github.com/marang/riotbox/pull/1135)`
- Merge commit: `03d9559c4db49f7061278d4bd53d9f48fd40c649`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-core daw_session_audible_output -- --nocapture; cargo test -p riotbox-app daw_export_report_surface_gate_tracks -- --nocapture; cargo test -p riotbox-core; cargo test -p riotbox-app; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `Implement actual DAW audible-output proof producer/capture before enabling export.daw_session.`

## Why This Ticket Existed

Reserve typed Session/Core QA gate for future DAW audible-output proof without overclaiming playback/export readiness.

## What Shipped

- Added daw_session_audible_output_proof QA gate helper; projected passed/failed audible proof through DAW session surface blockers; documented disabled-export boundary.

## Notes

- Passing audible-output proof removes only audible_output_proof_missing. developer_proof_only and daw_writer_missing remain blockers; no DAW writer, host runner, audio capture, live device dependency, or runnable export was added.
