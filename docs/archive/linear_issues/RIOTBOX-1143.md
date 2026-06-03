# `RIOTBOX-1143` P016: Add DAW session manifest/proof payload contracts

- Ticket: `RIOTBOX-1143`
- Title: `P016: Add DAW session manifest/proof payload contracts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1143/p016-add-daw-session-manifestproof-payload-contracts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1143-p016-add-daw-session-manifestproof-payload-contracts`
- Linear branch: `feature/riotbox-1143-p016-add-daw-session-manifestproof-payload-contracts`
- Assignee: `Markus`
- Labels: None
- PR: `#1122 (https://github.com/marang/riotbox/pull/1122)`
- Merge commit: `226f8baf06e13def7a71ff5dac2efc9065b54805`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-core daw_session -- --nocapture`; `cargo test -p riotbox-core`; `cargo test -p riotbox-app`; `scripts/run_compact.sh /tmp/riotbox-1143-just-ci.log just ci`; `GitHub rust-ci pass on PR #1122`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Adds typed DAW-session manifest/proof payload contracts behind the read-only writer plan, so future DAW writer work has deterministic JSON shapes, normalized hashes, and strict validation without claiming runnable export.

## What Shipped

- Added riotbox-core::daw_session_manifest::DawSessionManifest with stable schema id/version, receipt-derived placement refs, tempo-map ref, source JSON identities, planned DAW JSON identities, and normalized JSON/hash helpers.
- Added riotbox-core::daw_session_proof::DawSessionProof built from the manifest hash and the same DAW evidence bundle.
- Added strict constructor/from-receipt validation for DAW scope/boundary, unsupported DAW flag, missing placement/tempo evidence, missing source or planned JSON identities, wrong media types, and direct proof input validation.
- Documented the payload contract as schema/proof-input only: no filesystem writes, no Session mutation, no CLI writer, no runnable export.daw_session, and no audio output.

## Notes

- Branch review found and fixed a validation gap where DawSessionProof::new trusted direct identity inputs too much.
- Not audio-producing; structured listening review did not apply.
