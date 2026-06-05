# `RIOTBOX-1217` Add rebuild-only proof for professional-output packs

- Ticket: `RIOTBOX-1217`
- Title: `Add rebuild-only proof for professional-output packs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1217/add-rebuild-only-proof-for-professional-output-packs`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1217-add-rebuild-only-proof-for-professional-output-packs`
- Linear branch: `feature/riotbox-1217-add-rebuild-only-proof-for-professional-output-packs`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1192 (https://github.com/marang/riotbox/pull/1192)`
- Merge commit: `59560337ad0c0f111c152ef214b89d480afab613`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py scripts/route_weak_output_fixes.py`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1217-dense-smoke-3`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1217-matrix-smoke-1`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1217-source-wav-smoke-1`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1217-suite-smoke-1`; `scripts/run_compact.sh /tmp/riotbox-1217-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-1217-just-ci.log just ci`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1217-suite-smoke-review`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Professional-output diagnostics still included raw source-layer material, which could mask weak Riotbox-generated or rebuilt output. Riotbox needed source-layer-off proof so a diagnostic pass demonstrates that the box itself carries useful sound when the original source bed is removed.

## What Shipped

- Added a source_layer_gain render path and wrote 06_rebuild_only_performance.wav in the dense-break performance pack.
- Disabled raw source bleed for the rebuild-only render while preserving source-derived chops, transient snaps, drums, bass pressure, dropout, and restore.
- Added rebuild-only proof metrics for rebuild/full RMS, rebuild/source RMS, rebuild/source correlation, source-on/rebuild-only correlation, rebuild pressure/hook, and rebuild restore/pressure.
- Rejected silent, weak, source-masked, identical, or role-weak rebuild-only output through dense pack failure codes and CI gates.
- Propagated rebuild-only proof into pro-pressure source matrix, professional source WAV pack, and professional output suite key metrics.
- Updated docs and roadmap to keep rebuild-only evidence diagnostic with quality_proof false and human_verdict unverified.

## Notes

- The source-layer-off diagnostic removes raw source bleed, not source-derived source transformations; it is stronger diagnostic QA, not final release-grade rebuild proof.
