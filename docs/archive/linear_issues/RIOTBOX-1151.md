# `RIOTBOX-1151` P016: Add guarded DAW JSON package execute CLI proof path

- Ticket: `RIOTBOX-1151`
- Title: `P016: Add guarded DAW JSON package execute CLI proof path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1151/p016-add-guarded-daw-json-package-execute-cli-proof-path`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1151-p016-add-guarded-daw-json-package-execute-cli-proof-path`
- Linear branch: `feature/riotbox-1151-p016-add-guarded-daw-json-package-execute-cli-proof-path`
- Assignee: `Markus`
- Labels: None
- PR: `#1130 (https://github.com/marang/riotbox/pull/1130)`
- Merge commit: `7c6ecd26`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; cargo test -p riotbox-app daw_session_json_package_execute -- --nocapture; just daw-session-json-package-execute-smoke; cargo test -p riotbox-app; cargo test -p riotbox-core; scripts/run_compact.sh /tmp/riotbox-1151-just-ci-final.log just ci; GitHub rust-ci green`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `Next P016 DAW export slice: explicit Session receipt evidence attach/observer lifecycle or DAW host/import proof remains separate.`

## Why This Ticket Existed

P016 needed the existing DAW JSON package writer exposed as an explicit operator proof through the real binary without making DAW export runnable.

## What Shipped

- Added --daw-session-json-package-execute, package write/report summary, real-binary smoke, Just target, docs, and file-size split for launch_summary/parser tests.

## Notes

- No Session mutation, observer lifecycle, DAW host project/session file, host import proof, or audible output claim was added.
