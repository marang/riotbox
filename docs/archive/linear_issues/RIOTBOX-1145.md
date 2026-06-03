# `RIOTBOX-1145` P016: Add DAW session tempo-map payload contract

- Ticket: `RIOTBOX-1145`
- Title: `P016: Add DAW session tempo-map payload contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1145/p016-add-daw-session-tempo-map-payload-contract`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1145-p016-add-daw-session-tempo-map-payload-contract`
- Linear branch: `feature/riotbox-1145-p016-add-daw-session-tempo-map-payload-contract`
- Assignee: `Markus`
- Labels: None
- PR: `#1124 (https://github.com/marang/riotbox/pull/1124)`
- Merge commit: `33ec37c4bbda197f89ba6ca999b65afd8aa70955`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-core daw_session_tempo_map -- --nocapture`; `cargo test -p riotbox-core daw_session -- --nocapture`; `cargo test -p riotbox-core`; `scripts/run_compact.sh /tmp/riotbox-1145-just-ci-final.log just ci`; `GitHub PR #1124 rust-ci green`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Close the missing tempo_map.json Core payload contract before any DAW session JSON writer emits the planned package layout.

## What Shipped

- Added riotbox_core::daw_session_tempo_map::DawSessionTempoMap with schema id/version, package id, DAW receipt/action identity, source/timing hypothesis refs, beat range, and BPM micros.
- Added from_receipt validation for DAW scope, boundary, role, unsupported DAW flag, missing/blank source refs, non-advancing beat range, invalid tempo, and blank package id.
- Added deterministic normalized JSON bytes/hash helpers and tests proving stable roundtrip plus tempo-sensitive hash changes.
- Updated specs to document manifest/tempo-map/proof payload contracts as schema-only, non-writing, non-runnable export groundwork.

## Notes

- Branch review used code-review and code-review-rust; added direct blank package_id validation coverage before PR.
- Not audio-producing work; structured listening review not applicable.
