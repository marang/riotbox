# `RIOTBOX-1106` P016: Add stem-package manifest proof hash helper

- Ticket: `RIOTBOX-1106`
- Title: `P016: Add stem-package manifest proof hash helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1106/p016-add-stem-package-manifest-proof-hash-helper`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1106-p016-add-stem-package-manifest-proof-hash-helper`
- Linear branch: `feature/riotbox-1106-p016-add-stem-package-manifest-proof-hash-helper`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1094 (https://github.com/marang/riotbox/pull/1094)`
- Merge commit: `48ed2af2ca6bffb06704b626b711124c3e4b3c1c`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt -> pass`; `cargo test -p riotbox-core stem_package_manifest -> pass`; `cargo test -p riotbox-core -> pass`; `scripts/run_compact.sh /tmp/riotbox-1106-just-ci.log just ci -> pass`; `GitHub Actions: rust-ci on PR #1094 -> pass`
- Docs touched: `crates/riotbox-core/src/stem_package_manifest.rs`, `crates/riotbox-core/src/stem_package_manifest_tests.rs`, `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 needs deterministic stem-package manifest proof identity before a package writer can safely emit reproducible manifest/proof artifacts. The helper had to reuse the normalized JSON byte path so future proof hashes cannot drift through a parallel serializer.

## What Shipped

- Added StemPackageManifest::normalized_json_sha256() to compute SHA-256 from normalized_json_bytes().
- Added tests proving repeated hash stability, direct byte-hash equivalence, SHA-256 shape, and changed stem artifact identity changing the hash.
- Updated Session and Audio QA specs to clarify the helper is proof identity only, not a package writer or runnable stem export claim.

## Notes

- No audible behavior changed; structured listening review was not applicable.
