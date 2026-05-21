# `RIOTBOX-889` Improve Beat20 real-source downbeat confidence without relaxing fallback safety

- Ticket: `RIOTBOX-889`
- Title: `Improve Beat20 real-source downbeat confidence without relaxing fallback safety`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-889/improve-beat20-real-source-downbeat-confidence-without-relaxing`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-889-beat20-downbeat-confidence`
- Linear branch: `feature/riotbox-889-improve-beat20-real-source-downbeat-confidence-without`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`, `timing`
- PR: `#883 (https://github.com/marang/riotbox/pull/883)`
- Merge commit: `497a70e3261b7187d703c79585886e432969836e`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; git diff --check; py_compile source timing validators/report scripts; cargo test source_timing_probe/readiness/downbeat evidence; cargo test -p riotbox-audio source_timing_probe_/feral_grid_pack; just source-timing-grid-use-contract-fixtures; just listening-manifest-validator-fixtures; just source-timing-probe-json-validator-fixtures; just generated source timing probe smoke fixtures; just observer-audio-summary-validator-fixtures; just p012-all-lane-source-grid-output-proof; just source-timing-example-probe-report-local; just ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md; docs/benchmarks/source_timing_example_probe_report_2026-05-21.md`
- Follow-ups: `None`

## Why This Ticket Existed

Surface downbeat phase margin evidence so Beat20-style ambiguous loops explain why they stay manual-confirm instead of only showing a low score.

## What Shipped

- Added primary_downbeat_margin to source_timing_probe JSON/text output and Feral grid manifests, updated validators/fixtures/proof summaries, and preserved Beat20 as manual_confirm_only with a tiny 0.0053614676 phase margin.

## Notes

- Beat20 remained ambiguous/manual_confirm_only; Beat08/Beat03/DH_BeatC stayed stable with larger margins.
