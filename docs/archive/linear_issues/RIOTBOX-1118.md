# `RIOTBOX-1118` P016: Add CI-safe stem-package proof fixture

- Ticket: `RIOTBOX-1118`
- Title: `P016: Add CI-safe stem-package proof fixture`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1118/p016-add-ci-safe-stem-package-proof-fixture`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1118-p016-add-ci-safe-stem-package-proof-fixture`
- Linear branch: `feature/riotbox-1118-p016-add-ci-safe-stem-package-proof-fixture`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`
- PR: `#1097 (https://github.com/marang/riotbox/pull/1097)`
- Merge commit: `b932eaeb2d8a3efef942c31fe4818d690953f5f9`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-core stem_package_manifest_fixture_roundtrips_json_and_keeps_readiness_blocked; cargo test -p riotbox-core; scripts/run_compact.sh /tmp/riotbox-1118-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/session_file_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 needs a deterministic CI-safe bridge proving stem-package receipt, manifest, and proof JSON contract wiring before any musician-ready package writer is claimed.

## What Shipped

- Extended the in-memory receipt fixture to derive and roundtrip StemPackageProof JSON, assert stable proof serialization, verify manifest SHA linkage, and keep stem-package readiness blocked for unsupported/deferred scope.

## Notes

- No audible behavior changed; structured listening review was not applicable.
