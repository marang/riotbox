# `RIOTBOX-1117` P016: Build stem-package proof from manifest evidence

- Ticket: `RIOTBOX-1117`
- Title: `P016: Build stem-package proof from manifest evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1117/p016-build-stem-package-proof-from-manifest-evidence`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1117-p016-build-stem-package-proof-from-manifest-evidence`
- Linear branch: `feature/riotbox-1117-p016-build-stem-package-proof-from-manifest-evidence`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1096 (https://github.com/marang/riotbox/pull/1096)`
- Merge commit: `2ae298e360c247c74bf1a6054d98530e8c6882f3`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt -> pass`; `cargo test -p riotbox-core stem_package_proof -> pass`; `cargo test -p riotbox-core -> pass`; `scripts/run_compact.sh /tmp/riotbox-1117-just-ci.log just ci -> pass`; `GitHub Actions: rust-ci on PR #1096 -> pass`
- Docs touched: `crates/riotbox-core/src/stem_package_proof.rs`, `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

After defining the stem-package proof schema, P016 needed a narrow bridge from the typed manifest to the typed proof payload. The helper had to use normalized_json_sha256 so future proof artifacts cannot drift through a second serializer path.

## What Shipped

- Added StemPackageProof::from_manifest(&StemPackageManifest).
- Derived package, receipt/action, claimed role, and JSON identity fields from manifest evidence.
- Used StemPackageManifest::normalized_json_sha256 for manifest identity.
- Added tests proving changed manifest artifact identity changes the derived proof manifest hash.
- Updated specs to clarify the helper is proof identity wiring only, not file writing or export readiness.

## Notes

- No audible behavior changed; structured listening review was not applicable.
