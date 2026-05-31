# `RIOTBOX-1075` Add source and capture lineage refs to export artifact evidence

- Ticket: `RIOTBOX-1075`
- Title: `Add source and capture lineage refs to export artifact evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1075/add-source-and-capture-lineage-refs-to-export-artifact-evidence`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1075-artifact-lineage-refs`
- Linear branch: `feature/riotbox-1075-add-source-and-capture-lineage-refs-to-export-artifact`
- Assignee: `Markus`
- Labels: None
- PR: `#1052 (https://github.com/marang/riotbox/pull/1052)`
- Merge commit: `2c5a680adfe630a591b9833dbbc506db3a3de67d`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core session::export_types -- --nocapture`; `cargo test -p riotbox-core replay::export_receipt -- --nocapture`; `cargo test -p riotbox-app export_receipt_hydration_preflight -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1052`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1077`

## Why This Ticket Existed

Export artifact evidence needs typed source and capture lineage before stems, DAW packages, or resample-derived export scopes can be claimed.

## What Shipped

- Added optional source_graph_ref, source_capture_refs, and lineage_capture_refs to export artifact-set entries.
- Preserved product-mix receipt compatibility with empty/default lineage evidence.
- Preserved lineage evidence through export receipt replay validation plans.
- Updated specs and focused serialization/replay/preflight tests.

## Notes

- No stem writing, DAW export, live recording export, or lineage QA enforcement shipped in this slice.
