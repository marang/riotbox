# `RIOTBOX-1138` P016: Add stem-package operator report CLI smoke proof

- Ticket: `RIOTBOX-1138`
- Title: `P016: Add stem-package operator report CLI smoke proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1138/p016-add-stem-package-operator-report-cli-smoke-proof`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1138-p016-add-stem-package-operator-report-cli-smoke-proof`
- Linear branch: `feature/riotbox-1138-p016-add-stem-package-operator-report-cli-smoke-proof`
- Assignee: `Markus`
- Labels: None
- PR: `#1116 (https://github.com/marang/riotbox/pull/1116)`
- Merge commit: `0a961e15b3c183c568fe2ef2c70c6880fda179ab`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check; git diff --check; just stem-package-local-ci-report-smoke; cargo test -p riotbox-app; scripts/run_compact.sh /tmp/riotbox-1138-just-ci.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/session_file_spec.md; docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1130 remains Todo for musician-facing stem-package surfacing gate; RIOTBOX-1036 remains P016 backlog anchor for full arrangement/stem/live export workflow.`

## Why This Ticket Existed

The stem-package operator report had unit coverage but no explicit repo-level smoke command proving the ready and missing-file report path through the actual CLI.

## What Shipped

- Added a real-binary integration smoke for stem-package local CI execute/report, added just stem-package-local-ci-report-smoke, and documented the smoke as internal CI/operator proof rather than final DAW export readiness.

## Notes

- The smoke creates all files in a temp directory, removes one stem to prove missing_local_files, and does not require or claim human listening because it validates report plumbing rather than audible output.
