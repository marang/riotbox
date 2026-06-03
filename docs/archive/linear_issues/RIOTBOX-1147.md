# `RIOTBOX-1147` P016: Add CI-safe DAW session JSON writer proof

- Ticket: `RIOTBOX-1147`
- Title: `P016: Add CI-safe DAW session JSON writer proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1147/p016-add-ci-safe-daw-session-json-writer-proof`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1147-p016-add-ci-safe-daw-session-json-writer-proof`
- Linear branch: `feature/riotbox-1147-p016-add-ci-safe-daw-session-json-writer-proof`
- Assignee: `Markus`
- Labels: None
- PR: `#1126 (https://github.com/marang/riotbox/pull/1126)`
- Merge commit: `46220ea2ee4e24f3921d32c91163cfbeed87dfa1`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `just daw-session-json-writer-smoke`; `cargo test -p riotbox-app daw_session_writer_plan -- --nocapture`; `cargo test -p riotbox-core daw_session -- --nocapture`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core`; `scripts/run_compact.sh /tmp/riotbox-1147-just-ci-final.log just ci`; `GitHub PR #1126 rust-ci green`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prove the first DAW session package file-emission path without claiming full DAW export, host import, observer lifecycle, or musician-facing export readiness.

## What Shipped

- Added internal write_daw_session_json_package for manifest, tempo-map, and proof JSON emission gated by the existing writer plan.
- Used staging under the requested destination and promoted the final daw_session/ package only after validation.
- Hashed final manifest/tempo/proof JSON and verified proof manifest hash linkage.
- Rejected blocked plans and existing final package directories without mutating Session.
- Added just daw-session-json-writer-smoke covering explicit write path, final files, hash linkage, no Session mutation, existing-final rejection, and missing-source blocking.
- Updated specs to separate CI-safe JSON writer proof from full DAW export, host import, observer lifecycle, DAW audio, and runnable export.daw_session.

## Notes

- Branch review used code-review and code-review-rust; no blocking findings remained before PR.
- Writes JSON only in explicit CI proof path; not audio-producing, listening review not applicable.
