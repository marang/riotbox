# `RIOTBOX-1162` P016: Require DAW writer proof before host-import proof can pass

- Ticket: `RIOTBOX-1162`
- Title: `P016: Require DAW writer proof before host-import proof can pass`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1162/p016-require-daw-writer-proof-before-host-import-proof-can-pass`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1162-p016-require-daw-writer-proof-before-host-import-proof-can-pass`
- Linear branch: `feature/riotbox-1162-p016-require-daw-writer-proof-before-host-import-proof-can`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1141 (https://github.com/marang/riotbox/pull/1141)`
- Merge commit: `982ec1b5514e0a21370b059ca4a566242f29ef7c`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app --bin riotbox-app daw_session_host_import_proof_cli -- --nocapture`; `cargo test -p riotbox-app --test daw_session_host_import_proof_apply_smoke -- --nocapture`; `cargo test -p riotbox-app`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Keep DAW host-import proof ordered after writer proof on the same receipt.

## What Shipped

- Host-import proof apply now records daw_writer_proof_missing and a failed gate when writer proof is absent; ordered writer-proof receipts can pass; CLI/smoke tests and specs updated.

## Notes

- None
