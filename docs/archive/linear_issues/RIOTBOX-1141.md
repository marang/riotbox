# `RIOTBOX-1141` P016: Add DAW export operator readiness report surface

- Ticket: `RIOTBOX-1141`
- Title: `P016: Add DAW export operator readiness report surface`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1141/p016-add-daw-export-operator-readiness-report-surface`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1141-p016-add-daw-export-operator-readiness-report-surface`
- Linear branch: `feature/riotbox-1141-p016-add-daw-export-operator-readiness-report-surface`
- Assignee: `Markus`
- Labels: None
- PR: `#1120 (https://github.com/marang/riotbox/pull/1120)`
- Merge commit: `72f3a3a6b10b978da398ec215baccb934be3ba01`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-app daw_export_report -- --nocapture`; `just daw-export-readiness-report-smoke`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core`; `scripts/run_compact.sh /tmp/riotbox-1141-just-ci-2.log just ci`; `GitHub rust-ci pass on PR #1120`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 needed a compact read-only DAW export operator report so placement, tempo-map, file availability, unsupported DAW boundary, and missing writer blockers are visible without making DAW export runnable.

## What Shipped

- Added riotbox-app --daw-export-readiness-report --session <session.json> as a read-only JSON report over the latest daw_session receipt.
- Derived report status from Session receipt truth, Core arrangement placement and tempo-map readiness validators, and existing artifact preflight ordering.
- Separated readiness blockers from fixed release blockers so ready_for_writer still reports developer_proof_only and daw_writer_missing instead of claiming musician-facing DAW export completion.
- Added parser/unit coverage plus a real-binary smoke test through just daw-export-readiness-report-smoke.

## Notes

- Not audio-producing; structured listening review was not applicable and no DAW files are written.
