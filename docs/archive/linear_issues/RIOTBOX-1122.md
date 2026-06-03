# `RIOTBOX-1122` P016: Resolve stem-package manifest/proof non-circular identity

- Ticket: `RIOTBOX-1122`
- Title: `P016: Resolve stem-package manifest/proof non-circular identity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1122/p016-resolve-stem-package-manifestproof-non-circular-identity`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1122-p016-resolve-stem-package-manifestproof-non-circular`
- Linear branch: `feature/riotbox-1122-p016-resolve-stem-package-manifestproof-non-circular`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1101 (https://github.com/marang/riotbox/pull/1101)`
- Merge commit: `53b9a60d0dcdef63e049d8c9b9c3fb5621a4b8ec`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-core stem_package_manifest`; `cargo test -p riotbox-core stem_package_proof`; `cargo test -p riotbox-core`; `scripts/run_compact.sh /tmp/riotbox-1122-just-ci.log just ci`; `GitHub rust-ci pass`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The stem-package writer plan exposed a self-reference risk: manifest/proof payloads carried JSON artifact SHA fields even though those file hashes should be produced from the final written payload bytes. P016 needed this resolved before any package writer could honestly claim deterministic proof readiness.

## What Shipped

- Removed embedded SHA-256 from typed manifest/proof JSON payload identities; those identities now carry only role, location, and media type.
- Kept written manifest/proof JSON file hashes as receipt artifact-set truth after the files exist.
- Added Core regressions proving receipt-side JSON file SHA changes do not change manifest/proof payload hashes, while stem artifact identity remains hash-sensitive.
- Updated action, session, and audio-QA specs with the non-circular manifest/proof identity rule.

## Notes

- No audible behavior changed; structured listening review was not applicable.
