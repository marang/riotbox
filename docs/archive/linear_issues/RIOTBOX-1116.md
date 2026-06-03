# `RIOTBOX-1116` P016: Define stem-package proof JSON schema type

- Ticket: `RIOTBOX-1116`
- Title: `P016: Define stem-package proof JSON schema type`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1116/p016-define-stem-package-proof-json-schema-type`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1116-p016-define-stem-package-proof-json-schema-type`
- Linear branch: `feature/riotbox-1116-p016-define-stem-package-proof-json-schema-type`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1095 (https://github.com/marang/riotbox/pull/1095)`
- Merge commit: `cf0dce4dd77e5ddf3154e117728797e37fbcf02c`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt -> pass`; `cargo test -p riotbox-core stem_package_proof -> pass`; `cargo test -p riotbox-core -> pass`; `scripts/run_compact.sh /tmp/riotbox-1116-just-ci.log just ci -> pass`; `GitHub Actions: rust-ci on PR #1095 -> pass`
- Docs touched: `crates/riotbox-core/src/stem_package_proof.rs`, `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 needed a typed stem-package proof JSON contract after adding deterministic manifest hashing. Without a proof schema, future package writers would have to invent loose strings or app-local proof payloads instead of reusing Session/Core receipt identity.

## What Shipped

- Added StemPackageProof with stable schema id/version and fields for package id, export scope, receipt/action ids, manifest SHA-256, claimed roles, and manifest/proof JSON identities.
- Added validation for blank package id, blank manifest SHA-256, non-stem or duplicate claimed roles, and invalid JSON identity shape.
- Added roundtrip, serialized-change, and rejection tests.
- Updated Session and Audio QA specs to clarify this is proof contract wiring only, not a writer or runnable stem export claim.

## Notes

- No audible behavior changed; structured listening review was not applicable.
