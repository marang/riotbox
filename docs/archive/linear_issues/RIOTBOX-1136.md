# `RIOTBOX-1136` P016: Add stem-package operator proof summary command

- Ticket: `RIOTBOX-1136`
- Title: `P016: Add stem-package operator proof summary command`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1136/p016-add-stem-package-operator-proof-summary-command`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1136-p016-add-stem-package-operator-proof-summary-command`
- Linear branch: `feature/riotbox-1136-p016-add-stem-package-operator-proof-summary-command`
- Assignee: `Markus`
- Labels: None
- PR: `#1114 (https://github.com/marang/riotbox/pull/1114)`
- Merge commit: `7f7a3bfc3e51bb341f113708c5106657ebcad08f`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app --bin riotbox-app stem_package -- --nocapture`; `cargo test -p riotbox-app`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1136-just-ci.log just ci`; `GitHub rust-ci PR #1114`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The stem-package path could write a guarded internal package proof, but operators had no bounded read-only command to inspect whether an existing Session receipt still matched local package files. This ticket existed to expose receipt QA status, artifact availability, and missing-file diagnostics without regenerating stems or presenting the proof as final musician export UX.

## What Shipped

- Added --stem-package-local-ci-report --session <session.json> as a read-only operator proof summary command.
- Reported latest stem-package receipt metadata, stem roles, manifest/proof identities, QA gate status, Core readiness blockers, package directory, local artifact availability, and missing local files.
- Rejected report arguments that could write or imply regeneration, including observer, destination, role, source, graph, sidecar, and seed arguments.
- Kept product-mix receipts explicitly separate by reporting no_stem_package_receipt for product-mix-only Sessions.

## Notes

- This remains an internal operator/developer proof report and does not make stem-package export a final musician-facing DAW workflow.
