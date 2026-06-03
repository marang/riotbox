# `RIOTBOX-1119` P016: Add reserved stem-package action contract types

- Ticket: `RIOTBOX-1119`
- Title: `P016: Add reserved stem-package action contract types`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1119/p016-add-reserved-stem-package-action-contract-types`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1119-p016-add-reserved-stem-package-action-contract-types`
- Linear branch: `feature/riotbox-1119-p016-add-reserved-stem-package-action-contract-types`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1098 (https://github.com/marang/riotbox/pull/1098)`
- Merge commit: `d0a3376f8a399c80ba3794c45278c91d287e14c6`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-core action::tests; cargo test -p riotbox-core; cargo test -p riotbox-app replay_warning_family_labels_cover_every_action_command; scripts/run_compact.sh /tmp/riotbox-1119-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md`
- Follow-ups: `RIOTBOX-1120, RIOTBOX-1121`

## Why This Ticket Existed

P016 needed a typed reserved export.stem_package Core action contract before queue and writer implementation so later slices do not branch on ad hoc strings or imply runnable stem export too early.

## What Shipped

- Added ActionCommand::ExportStemPackage, typed StemPackageExport params for scope/destination/claimed stem roles/policies/manifest inclusion, replay-unsupported coverage, Export warning-family classification, and Action Lexicon reserved-contract wording.

## Notes

- No audible behavior changed; structured listening review was not applicable.
