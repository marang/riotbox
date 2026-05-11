# `RIOTBOX-772` Add Source Timing grid-use classification to probe reports

- Ticket: `RIOTBOX-772`
- Title: `Add Source Timing grid-use classification to probe reports`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-772/add-source-timing-grid-use-classification-to-probe-reports`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-772-add-source-timing-grid-use-classification-to-probe-reports`
- Linear branch: `feature/riotbox-772-add-source-timing-grid-use-classification-to-probe-reports`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#766 (https://github.com/marang/riotbox/pull/766)`
- Merge commit: `60fe9433cb884e3ec5f97cd7ebab9535eff84e5b`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo test -p riotbox-audio --bin source_timing_probe -- --nocapture`; `just source-timing-probe-json-validator-fixtures`; `just generated-source-timing-probe-json-smoke`; `just generated-degraded-source-timing-probe-json-smoke`; `just generated-ambiguous-source-timing-probe-json-smoke`; `just source-timing-example-probe-report-fixtures`; `just source-timing-example-probe-report-local artifacts/audio_qa/local/source_timing_example_probe_report_expected.md`; `just ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed a compact machine-readable classification for whether Source Timing evidence is locked, short-loop usable with manual confirmation, manual-only, fallback, or unavailable instead of forcing QA and musicians to infer that from multiple raw fields.

## What Shipped

- Added grid_use to the Source Timing probe CLI JSON and text output.
- Surfaced grid_use in the example probe Markdown report and optional expectations checks.
- Validated the field in source timing probe JSON fixtures and generated probe smokes.
- Documented grid_use as a conservative QA classification, not a production arbitrary-audio detector claim.

## Notes

- No detector thresholds, lane behavior, or audio-producing behavior changed.
