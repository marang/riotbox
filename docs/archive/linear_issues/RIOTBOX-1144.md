# `RIOTBOX-1144` P016: Surface DAW session payload preview in writer plan

- Ticket: `RIOTBOX-1144`
- Title: `P016: Surface DAW session payload preview in writer plan`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1144/p016-surface-daw-session-payload-preview-in-writer-plan`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1144-p016-surface-daw-session-payload-preview-in-writer-plan`
- Linear branch: `feature/riotbox-1144-p016-surface-daw-session-payload-preview-in-writer-plan`
- Assignee: `Markus`
- Labels: None
- PR: `#1123 (https://github.com/marang/riotbox/pull/1123)`
- Merge commit: `cbc42413e086fdf05141274b21e01b4f2a33277f`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-app daw_session_writer_plan -- --nocapture`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core`; `just daw-session-writer-plan-smoke`; `scripts/run_compact.sh /tmp/riotbox-1144-just-ci-final.log just ci`; `GitHub PR #1123 rust-ci green`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Connect Core DAW session manifest/proof contracts to the read-only writer-plan report so the next writer step has deterministic payload identity before file emission.

## What Shipped

- Added read-only payload_preview with ready/blocked status, typed blockers, schema ids, planned manifest/proof paths, normalized manifest hash, and proof manifest hash.
- Mirrored no receipt, missing destination, missing local files, placement, tempo, artifact identity, and unreadable-file blockers into the payload preview without emitting hashes when blocked.
- Kept DAW session export non-runnable and non-mutating: no directories/files, observer events, or Session changes.
- Covered ready, no receipt, missing local files, and missing destination preview shapes in CLI/smoke tests.

## Notes

- Branch review used code-review and code-review-rust; fixed module-size review finding and added missing_destination_root preview coverage before PR.
- Not audio-producing work; structured listening review not applicable.
