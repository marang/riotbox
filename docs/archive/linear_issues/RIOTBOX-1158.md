# `RIOTBOX-1158` P016: Define first DAW session writer action boundary

- Ticket: `RIOTBOX-1158`
- Title: `P016: Define first DAW session writer action boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1158/p016-define-first-daw-session-writer-action-boundary`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1158-p016-define-first-daw-session-writer-action-boundary`
- Linear branch: `feature/riotbox-1158-p016-define-first-daw-session-writer-action-boundary`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1137 (https://github.com/marang/riotbox/pull/1137)`
- Merge commit: `5c482c35cece95cdf0113076ed7e929ef27515de`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; just ci (/tmp/riotbox-1158-just-ci.log); just ci (/tmp/riotbox-1158-just-ci-rerun.log); GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1159 adds the first CI-safe DAW session writer proof skeleton on top of the reserved boundary.`

## Why This Ticket Existed

Define the first DAW session writer/action boundary before code can safely remove daw_writer_missing or expose export.daw_session.

## What Shipped

- Documented queue, commit/side-effect, Session/replay, observer/user, undo, and QA requirements for the future daw_session.local_project_writer_v1 writer boundary; clarified one-gate-at-a-time blocker removal and stem-package field ownership.

## Notes

- Contract-only slice; no ActionCommand, CLI writer, observer lifecycle, host runner, audio capture, DAW project/session file emission, or runnable export.daw_session path was added.
