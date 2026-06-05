# `RIOTBOX-1221` Add source-derived bass-pressure movement proof for sparse sources

- Ticket: `RIOTBOX-1221`
- Title: `Add source-derived bass-pressure movement proof for sparse sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1221/add-source-derived-bass-pressure-movement-proof-for-sparse-sources`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1221-add-source-derived-bass-pressure-movement-proof-for-sparse`
- Linear branch: `feature/riotbox-1221-add-source-derived-bass-pressure-movement-proof-for-sparse`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1196 (https://github.com/marang/riotbox/pull/1196)`
- Merge commit: `c0f6618b44319ba5df8ee87ee555fbed5a4f1c30`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_output_suite.py; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1221-source-wav-smoke-review; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md; docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1222 source-derived hook/chop selection proof; RIOTBOX-1223 pad/noise gated texture improvement`

## Why This Ticket Existed

Sparse-bass pressure diagnostics could still sound like a fixed melodic contour; P022 needs source-derived movement proof before this path counts as stronger evidence.

## What Shipped

- Derived sparse-bass pressure frequencies from source low-band energy, timing centroid, and transient content; surfaced source-derived movement, fixed-contour distance, and frequency-span metrics through the pro-pressure source matrix, professional source WAV pack, and professional output suite; added validator and smoke negative fixtures for non-source-derived or collapsed fixed-contour sparse movement; documented the evidence boundary in benchmarks, roadmap, and RBX-072.

## Notes

- None
