# `RIOTBOX-1131` P016: Commit local CI stem-package writer through export action

- Ticket: `RIOTBOX-1131`
- Title: `P016: Commit local CI stem-package writer through export action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1131/p016-commit-local-ci-stem-package-writer-through-export-action`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1131-p016-commit-local-ci-stem-package-writer-through-export`
- Linear branch: `feature/riotbox-1131-p016-commit-local-ci-stem-package-writer-through-export`
- Assignee: `Markus`
- Labels: None
- PR: `#1109 (https://github.com/marang/riotbox/pull/1109)`
- Merge commit: `1a3ccbc154a4f65a6c4fd7d65336be7eafe98b00`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-app local_ci_stem_package_export; cargo test -p riotbox-app product_mix_export; cargo test -p riotbox-app --bin riotbox-app export_stem_package_observer; cargo test -p riotbox-app; cargo test -p riotbox-core; git diff --check; scripts/run_compact.sh /tmp/riotbox-1131-just-ci.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `Continue P016 with RIOTBOX-1132 CLI dry-run surface, RIOTBOX-1133 Jam inspect surfacing, and RIOTBOX-1134 restore diagnostics.`

## Why This Ticket Existed

The local stem-package writer proof needed to commit through the real export action, queue/history, Session receipt, commit-record, and observer lifecycle path instead of remaining a standalone file-emission helper.

## What Shipped

- Added local_ci_package_v1 as an explicit stem-package action boundary; committed deterministic local CI drums/bass stem packages through JamAppState; rejected unsupported roles without receipts; proved product-mix unchanged and observer completed lifecycle from the real committed writer path.

## Notes

- Internal proof boundary only: no musician-facing UI/Ghost/CLI stem export control yet; human_verdict remains unverified for this slice.
