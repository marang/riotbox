# `RIOTBOX-1132` P016: Add CLI dry-run surface for local stem-package proof

- Ticket: `RIOTBOX-1132`
- Title: `P016: Add CLI dry-run surface for local stem-package proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1132/p016-add-cli-dry-run-surface-for-local-stem-package-proof`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1132-p016-add-cli-dry-run-surface-for-local-stem-package-proof`
- Linear branch: `feature/riotbox-1132-p016-add-cli-dry-run-surface-for-local-stem-package-proof`
- Assignee: `Markus`
- Labels: None
- PR: `#1110 (https://github.com/marang/riotbox/pull/1110)`
- Merge commit: `9f1054bde31d10d19d5026c9f12234dfe2e94d3c`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app --bin riotbox-app stem_package_export_cli`; `cargo test -p riotbox-app --bin riotbox-app`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core stem_package_writer`; `cargo test -p riotbox-core`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1132-just-ci.log just ci`; `GitHub rust-ci pass on PR #1110`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-1132 existed to give the internal P016 stem-package writer a safe CLI planning surface before any musician-facing export command exists. The slice needed to show destination paths, supported role claims, and blockers without writing files or mutating Session receipts.

## What Shipped

- Added a stem-package local CI dry-run CLI mode that requires an explicit destination and claimed stem roles.
- Reported JSON readiness, supported and unsupported roles, blockers, planned final artifact paths, and writes_files:false.
- Kept the dry-run separated from source/session/observer launch modes so product-mix and normal app launch behavior stay unchanged.
- Reused the core local CI writer plan and documented the dry-run as developer planning evidence, not musician-ready DAW export or audio QA proof.

## Notes

- No structured listening review applies: this is a no-write CLI planning surface, not an audible output change.
