# `RIOTBOX-1296` Strengthen sparse bass-pressure movement proof

- Ticket: `RIOTBOX-1296`
- Title: `Strengthen sparse bass-pressure movement proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1296/strengthen-sparse-bass-pressure-movement-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-19`
- Branch: `feature/riotbox-1296-sparse-bass-pressure-movement`
- Linear branch: `feature/riotbox-1296-strengthen-sparse-bass-pressure-movement-proof`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1270 (https://github.com/marang/riotbox/pull/1270)`
- Merge commit: `156d69c4e96f14f9a9c408d4a539f3d4b2cd2cd6`
- Deleted from Linear: `2026-06-28`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_output_suite.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/validate_professional_output_suite_contract.py; git diff --check; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1296-source-wav; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1296-matrix; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1296-suite; just ci; GitHub rust-ci pass on PR #1270`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Sparse bass-pressure diagnostics needed explicit low-band versus midrange proof so moving MC-202 material cannot pass as bass pressure unless the rendered pressure section actually carries low-band weight.

## What Shipped

- Added sparse_pressure_low_band_share and sparse_pressure_low_to_mid_ratio proof fields; propagated them through Source-WAV, pressure Source Matrix, and professional-output suite validation; added mutation fixtures for low-band-share weakness and midrange-phrase collapse; documented that the gates remain diagnostic and not human musical approval.

## Notes

- Evidence remains diagnostic with quality_proof false until structured listening review accepts the result. The thresholds use the script's existing band_energy_ratios amplitude-share metric, not an external power-spectrum ratio.
