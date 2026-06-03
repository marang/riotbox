# `RIOTBOX-1137` P016: Add stem-package-specific receipt boundary and profile identity

- Ticket: `RIOTBOX-1137`
- Title: `P016: Add stem-package-specific receipt boundary and profile identity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1137/p016-add-stem-package-specific-receipt-boundary-and-profile-identity`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1137-p016-add-stem-package-specific-receipt-boundary-and-profile`
- Linear branch: `feature/riotbox-1137-p016-add-stem-package-specific-receipt-boundary-and-profile`
- Assignee: `Markus`
- Labels: None
- PR: `#1115 (https://github.com/marang/riotbox/pull/1115)`
- Merge commit: `dd83f8f5db01bb2bb80bfe5dbf6b769c303ed47d`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check; git diff --check; cargo test -p riotbox-core stem_package -- --nocapture; cargo test -p riotbox-app stem_package -- --nocapture; cargo test -p riotbox-app; scripts/run_compact.sh /tmp/riotbox-1137-just-ci.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/session_file_spec.md; docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1138 continues P016 operator report smoke proof; RIOTBOX-1130 remains Todo for package proof hardening.`

## Why This Ticket Existed

Stem-package local CI receipts still reused product-mix feral-grid-demo/full_grid_mix identity, making reports and replay unable to prove that a package manifest/stem bundle was distinct from a full-grid WAV export.

## What Shipped

- Added typed stem-package identity values stem-package-local-ci/package_manifest/stem_package.local_ci_package_v1; made writer receipts, manifests, proofs, CLI summaries, observer snapshots, and tests preserve that identity while keeping product-mix receipts unchanged.

## Notes

- Legacy stem-package receipt artifact_path/export_hash now point at the package manifest; actual stem WAVs, manifest, and proof JSON remain authoritative in artifact_set[]. Human listening was not applicable because this was identity/proof-surface work, not audible render behavior.
