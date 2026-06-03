# `RIOTBOX-1130` P016: Gate musician-facing stem-package export surfacing

- Ticket: `RIOTBOX-1130`
- Title: `P016: Gate musician-facing stem-package export surfacing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1130/p016-gate-musician-facing-stem-package-export-surfacing`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1130-p016-gate-musician-facing-stem-package-export-surfacing`
- Linear branch: `feature/riotbox-1130-p016-gate-musician-facing-stem-package-export-surfacing`
- Assignee: `Markus`
- Labels: None
- PR: `#1117 (https://github.com/marang/riotbox/pull/1117)`
- Merge commit: `99de1aa758683b9490b65e85dad21eecb6a389a1`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check; git diff --check; cargo test -p riotbox-app stem_package -- --nocapture; cargo test -p riotbox-app export_readiness -- --nocapture; cargo test -p riotbox-app; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md; docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-1139 arrangement export receipt placement contract skeleton`

## Why This Ticket Existed

Prevent the internal stem-package local CI proof from being surfaced as a finished musician DAW export.

## What Shipped

- Added a shared JamAppState stem-package musician surface gate with typed blockers; reserved export.stem_package attempts now reject with gate blockers and no receipt/files; Jam Inspect and observer snapshots project the same disabled surface gate; specs/tests preserve the distinction between local CI proof readiness and musician export readiness.

## Notes

- None
