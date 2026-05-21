# `RIOTBOX-879` Add Source Timing actionability to probe CLI summaries

- Ticket: `RIOTBOX-879`
- Title: `Add Source Timing actionability to probe CLI summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-879/add-source-timing-actionability-to-probe-cli-summaries`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-879-source-timing-probe-actionability`
- Linear branch: `feature/riotbox-879-add-source-timing-actionability-to-probe-cli-summaries`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#873 (https://github.com/marang/riotbox/pull/873)`
- Merge commit: `72e916157380b105fc94180b9244a88541adb9b1`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio --bin source_timing_probe; cargo clippy -p riotbox-audio --bin source_timing_probe -- -D warnings; just source-timing-probe-json-validator-fixtures; just source-timing-example-probe-report-fixtures; just source-timing-grid-use-contract-fixtures; git diff --check; just ci; GitHub Rust CI success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Generated Feral-grid manifests and observer/audio summaries carried Source Timing actionability, but the standalone source_timing_probe CLI still reported only cue/readiness/manual-confirm/grid-use.

## What Shipped

- Added Source Timing actionability to source_timing_probe JSON and text output, validated it against readiness/manual-confirm state, updated probe/report/grid-use fixtures, and moved probe CLI tests into a semantic child test module to keep the main Rust file under the review budget.

## Notes

- None
