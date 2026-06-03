# `RIOTBOX-1163` P016: Require DAW writer and host-import proofs before audible-output proof can pass

- Ticket: `RIOTBOX-1163`
- Title: `P016: Require DAW writer and host-import proofs before audible-output proof can pass`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1163/p016-require-daw-writer-and-host-import-proofs-before-audible-output`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1163-p016-require-daw-writer-and-host-import-proofs-before`
- Linear branch: `feature/riotbox-1163-p016-require-daw-writer-and-host-import-proofs-before`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1142 (https://github.com/marang/riotbox/pull/1142)`
- Merge commit: `0416df127e5b66c20ce9af52e3e9a8bc499a2bda`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app --bin riotbox-app daw_session_audible_output_proof_cli -- --nocapture`; `cargo test -p riotbox-app --test daw_session_audible_output_proof_apply_smoke -- --nocapture`; `cargo test -p riotbox-app`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Keep DAW audible-output proof ordered after writer and host-import proof on the same receipt.

## What Shipped

- Audible-output proof apply now records missing prerequisite blockers and a failed gate when writer or host-import proof is absent; ordered writer/host/audible receipts can pass; CLI/smoke tests and specs updated.

## Notes

- None
