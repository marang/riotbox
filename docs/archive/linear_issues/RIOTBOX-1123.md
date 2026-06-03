# `RIOTBOX-1123` P016: Require all stem-package receipt QA gates for readiness

- Ticket: `RIOTBOX-1123`
- Title: `P016: Require all stem-package receipt QA gates for readiness`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1123/p016-require-all-stem-package-receipt-qa-gates-for-readiness`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1123-p016-require-all-stem-package-readiness-gates`
- Linear branch: `feature/riotbox-1123-p016-require-all-stem-package-receipt-qa-gates-for-readiness`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1102 (https://github.com/marang/riotbox/pull/1102)`
- Merge commit: `5ff3e2bdb899e4a7cd3bc5a09e762b94fc4f415d`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-core stem_package_readiness`; `cargo test -p riotbox-core`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1123-just-ci.log just ci`; `GitHub rust-ci pass`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The existing stem-package readiness guard could report Ready from a passed artifact-set gate alone once the unsupported scope flag was removed, even though P016 had already defined hash-stability, non-silence, lineage, and fallback-comparison gates. The receipt-level guard needed to enforce the full readiness contract before any future writer could accidentally overclaim stem-package readiness.

## What Shipped

- Added typed blockers for missing, deferred, and failed hash-stability, non-silence, lineage, and fallback-comparison QA gates.
- Changed `validate_stem_package_receipt_readiness` to require all five stem-package gates to be present and passed before Ready.
- Expanded Core readiness tests across missing, deferred, failed, unsupported-scope, and all-passed combinations.
- Updated Session and Audio QA specs with the complete readiness-gate set.

## Notes

- No package writer or runnable `export.stem_package` action was added.
- No audible behavior changed; structured listening review was not applicable.
