# `RIOTBOX-1073` Add per-artifact audio metrics to export artifact sets

- Ticket: `RIOTBOX-1073`
- Title: `Add per-artifact audio metrics to export artifact sets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1073/add-per-artifact-audio-metrics-to-export-artifact-sets`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1073-export-artifact-audio-metrics`
- Linear branch: `feature/riotbox-1073-add-per-artifact-audio-metrics-to-export-artifact-sets`
- Assignee: `Markus`
- Labels: None
- PR: `#1049 (https://github.com/marang/riotbox/pull/1049)`
- Merge commit: `68aa672be3a161076766484c4b7e6cdb1dce6175`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core audio_metrics -- --nocapture`; `cargo test -p riotbox-core export_receipt -- --nocapture`; `cargo test -p riotbox-core stem_package -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1049`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1074`

## Why This Ticket Existed

Stem non-silence and fallback-collapse gates need a typed place for per-artifact audio evidence before they can enforce audible claims.

## What Shipped

- Added optional ExportArtifactAudioMetrics to export artifact-set entries.
- Kept existing product-mix receipts compatible with metrics unset.
- Added serialization and backward-compatibility tests for artifact entries with and without metrics.
- Updated Session File and Audio QA specs without claiming stem non-silence yet.

## Notes

- No stem files, stem package export action, non-silence claim, or fallback-collapse claim shipped.
