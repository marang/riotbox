# `RIOTBOX-1142` P016: Add DAW session writer plan skeleton

- Ticket: `RIOTBOX-1142`
- Title: `P016: Add DAW session writer plan skeleton`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1142/p016-add-daw-session-writer-plan-skeleton`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1142-p016-add-daw-session-writer-plan-skeleton`
- Linear branch: `feature/riotbox-1142-p016-add-daw-session-writer-plan-skeleton`
- Assignee: `Markus`
- Labels: None
- PR: `#1121 (https://github.com/marang/riotbox/pull/1121)`
- Merge commit: `e0c03a2bc5890439402f8a9c19341116085876a9`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-app daw_session_writer_plan -- --nocapture`; `just daw-session-writer-plan-smoke`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core`; `scripts/run_compact.sh /tmp/riotbox-1142-just-ci.log just ci`; `GitHub rust-ci pass on PR #1121`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Adds a deterministic read-only DAW session writer plan before the real DAW writer exists, so operator tooling can prove planned DAW package identities and blockers without pretending export is runnable.

## What Shipped

- Added app-level daw_session_writer_plan contract from latest daw_session receipt plus operator readiness.
- Added riotbox-app --daw-session-writer-plan --session <session.json> --daw-session-destination <dir> JSON CLI.
- Reported planned arrangement manifest, tempo-map, and DAW-session proof identities under daw_session/ plus source artifact refs, placement refs, tempo-map ref, readiness blockers, and daw_writer_missing.
- Kept the skeleton dry-run only: no destination directory creation, no file writes, no observer events, no Session mutation, and no runnable export.daw_session action.
- Tightened parser rejection so --daw-session-destination cannot be silently ignored outside writer-plan mode.
- Added unit/smoke coverage and docs for the skeleton contract.

## Notes

- Branch review found and fixed a parser bug where --daw-session-destination could be ignored by some non-writer modes.
- Not audio-producing; structured listening review did not apply.
