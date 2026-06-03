# `RIOTBOX-1127` P016: Add local stem-package writer skeleton boundary

- Ticket: `RIOTBOX-1127`
- Title: `P016: Add local stem-package writer skeleton boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1127/p016-add-local-stem-package-writer-skeleton-boundary`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1127-p016-add-local-stem-package-writer-skeleton-boundary`
- Linear branch: `feature/riotbox-1127-p016-add-local-stem-package-writer-skeleton-boundary`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1106 (https://github.com/marang/riotbox/pull/1106)`
- Merge commit: `afed4bdc1b80acb589c53cac17ad3479140c8f46`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-core stem_package_writer; cargo test -p riotbox-core; git diff --check; scripts/run_compact.sh /tmp/riotbox-1127-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/session_file_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `Continue RIOTBOX-1128 to add the CI-safe stem-package writer file-emission proof.`

## Why This Ticket Existed

P016 needed a code-level local writer skeleton for stem_package.local_ci_package_v1 before any file-emitting writer could safely land.

## What Shipped

- Added riotbox-core::stem_package_writer with explicit local writer boundary/request/plan types, final package artifact identity planning, drums/bass-only supported role validation, local-directory destination validation, no-side-effect tests, and spec notes.

## Notes

- No package files are written, no hashes or audio metrics are produced, no receipt is constructed, and export.stem_package remains non-runnable.
