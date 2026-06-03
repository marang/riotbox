# `RIOTBOX-1135` P016: Add guarded stem-package execute CLI proof path

- Ticket: `RIOTBOX-1135`
- Title: `P016: Add guarded stem-package execute CLI proof path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1135/p016-add-guarded-stem-package-execute-cli-proof-path`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1135-p016-add-guarded-stem-package-execute-cli-proof-path`
- Linear branch: `feature/riotbox-1135-p016-add-guarded-stem-package-execute-cli-proof-path`
- Assignee: `Markus`
- Labels: None
- PR: `#1113 (https://github.com/marang/riotbox/pull/1113)`
- Merge commit: `845534090f1b656ea6e5eb90c65521766970f0b2`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app --bin riotbox-app stem_package -- --nocapture`; `cargo test -p riotbox-app`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1135-just-ci.log just ci`; `GitHub rust-ci PR #1113`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1136`

## Why This Ticket Existed

The stem-package path had a committed writer and dry-run plan, but no guarded CLI execute proof that wrote files through the real action/receipt/session path. This ticket existed to prove the internal local CI stem export can be executed safely without pretending the final musician-facing export UX is done.

## What Shipped

- Added --stem-package-local-ci-execute with required existing Session, explicit destination, and explicit stem roles.
- Executed through commit_stem_package_export_local_ci_package so the action, commit record, ready receipt, written stem package, and saved Session stay tied together.
- Emitted JSON and optional non-interactive observer evidence for ready and blocked execute outcomes.
- Kept dry-run/product-mix behavior unchanged and blocked unsupported roles or existing package destinations without saving a ready receipt.

## Notes

- This is still an internal operator/developer proof path for drums/bass local CI stems, not the final musician DAW export workflow.
