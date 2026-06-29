# `RIOTBOX-1316` P023: Fix Feral-grid source-collapse across different source examples

- Ticket: `RIOTBOX-1316`
- Title: `P023: Fix Feral-grid source-collapse across different source examples`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1316/p023-fix-feral-grid-source-collapse-across-different-source-examples`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1316-p023-fix-feral-grid-source-collapse-across-different-source`
- Linear branch: `feature/riotbox-1316-p023-fix-feral-grid-source-collapse-across-different-source`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1290 (https://github.com/marang/riotbox/pull/1290)`
- Merge commit: `41f4869e1c55ad7aa68306229d01cc94bfe4e70b`
- Deleted from Linear: `2026-06-29`
- Verification: `GitHub rust-ci passed`; `cargo fmt`; `cargo test -p riotbox-audio --bin feral_grid_pack`; `cargo clippy --all-targets --all-features -- -D warnings`; `just feral-grid-render-diversity-fixtures`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1316-suite-after-split`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1316-readiness-after-split`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Different source examples could collapse into near-identical feral-grid rendered roles, which undermined source-backed product output.

## What Shipped

- Rendered-WAV cross-source diversity gate, source-responsive W-30/TR-909 variation, removal of full_performance overlay artifacts from active QA, and diverse generated test/example sources marked non-proof.

## Notes

- None
