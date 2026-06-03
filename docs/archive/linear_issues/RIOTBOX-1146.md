# `RIOTBOX-1146` P016: Surface DAW tempo-map payload preview in writer plan

- Ticket: `RIOTBOX-1146`
- Title: `P016: Surface DAW tempo-map payload preview in writer plan`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1146/p016-surface-daw-tempo-map-payload-preview-in-writer-plan`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1146-p016-surface-daw-tempo-map-payload-preview-in-writer-plan`
- Linear branch: `feature/riotbox-1146-p016-surface-daw-tempo-map-payload-preview-in-writer-plan`
- Assignee: `Markus`
- Labels: None
- PR: `#1125 (https://github.com/marang/riotbox/pull/1125)`
- Merge commit: `8028c60041209d61f3cb6af821f7a383d604784a`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-app daw_session_writer_plan -- --nocapture`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core daw_session -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-1146-just-ci-final.log just ci`; `GitHub PR #1125 rust-ci green`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Complete the read-only DAW writer-plan payload preview so all planned DAW JSON payloads expose deterministic schema/path/hash identity before file emission.

## What Shipped

- Added tempo_map preview with schema id/version, planned tempo_map.json path, and normalized JSON hash.
- Built DawSessionTempoMap only when the upstream writer plan is ready and kept blocked previews hashless with typed blockers.
- Extended CLI and smoke tests for ready and blocked tempo-map preview shapes.
- Updated specs to document manifest/tempo/proof preview as payload-shape proof only, not DAW export completion.

## Notes

- Branch review used code-review and code-review-rust; no blocking findings remained before PR.
- Not audio-producing work; structured listening review not applicable.
