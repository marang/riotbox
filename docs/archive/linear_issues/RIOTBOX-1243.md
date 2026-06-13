# `RIOTBOX-1243` P023: Strengthen W-30 hook/chop policy for dense and tonal sources

- Ticket: `RIOTBOX-1243`
- Title: `P023: Strengthen W-30 hook/chop policy for dense and tonal sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1243/p023-strengthen-w-30-hookchop-policy-for-dense-and-tonal-sources`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1243-p023-w30-hook-chop-policy`
- Linear branch: `feature/riotbox-1243-p023-strengthen-w-30-hookchop-policy-for-dense-and-tonal`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1218 (https://github.com/marang/riotbox/pull/1218)`
- Merge commit: `61081115`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py; just professional-source-wav-pack-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`
- Follow-ups: `Continue P023 with destructive gesture and mix-bus tickets RIOTBOX-1244 and RIOTBOX-1245.`

## Why This Ticket Existed

Weak-output routing ranked chop_policy highest because dense and tonal cases still lacked a memorable transformed W-30 hook/riff.

## What Shipped

- HookChopPolicy now carries multiple source-derived riff offsets; the W-30 hook/riff renderer uses them; dense/tonal reports expose and gate hook_chop_riff_unique_source_offset_count; professional matrix/source/suite reports aggregate the metric.

## Notes

- Artifacts remain diagnostic-only with human_verdict unverified and quality_proof false.
