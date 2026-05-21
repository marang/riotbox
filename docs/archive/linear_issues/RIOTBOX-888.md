# `RIOTBOX-888` Expose downbeat phase confidence in generated Feral-grid Source Timing manifests

- Ticket: `RIOTBOX-888`
- Title: `Expose downbeat phase confidence in generated Feral-grid Source Timing manifests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-888/expose-downbeat-phase-confidence-in-generated-feral-grid-source-timing`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-888-downbeat-phase-confidence-manifest`
- Linear branch: `feature/riotbox-888-expose-downbeat-phase-confidence-in-generated-feral-grid`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`, `ux`
- PR: `#882 (https://github.com/marang/riotbox/pull/882)`
- Merge commit: `411a7e19afdf0daed14990821dee6caaa856a897`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; python3 -m py_compile scripts/validate_listening_manifest_json.py scripts/validate_source_timing_grid_use_contract_fixtures.py scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py; cargo test -p riotbox-core source_timing_probe_readiness; cargo test -p riotbox-audio --bin source_timing_probe; cargo test -p riotbox-audio --bin feral_grid_pack; cargo test -p riotbox-app --bin observer_audio_correlate; just source-timing-grid-use-contract-fixtures; just listening-manifest-validator-fixtures; just observer-audio-summary-validator-fixtures; just p012-all-lane-source-grid-output-proof; just p012-all-lane-proof-summary artifacts/audio_qa/local/p012_all_lane_source_grid_output_proof_summary.md; just ci; git diff --check; GitHub Rust CI #2188 success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-889 improves Beat20 real-source downbeat confidence if evidence supports it`

## Why This Ticket Existed

Generated Feral-grid manifests exposed the selected downbeat offset but not the existing downbeat phase score or alternate-phase count, making bar-phase confidence less inspectable than raw probe output.

## What Shipped

- Source Timing readiness reports and generated Feral-grid manifests now preserve primary downbeat score and alternate downbeat phase count; manifest validators require the fields and the compact P012 proof summary displays them for Recipe 15 real-source rows.

## Notes

- None
