# `RIOTBOX-1219` Add pad/noise source-family policy instead of dense-break promotion

- Ticket: `RIOTBOX-1219`
- Title: `Add pad/noise source-family policy instead of dense-break promotion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1219/add-padnoise-source-family-policy-instead-of-dense-break-promotion`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1219-add-padnoise-source-family-policy-instead-of-dense-break`
- Linear branch: `feature/riotbox-1219-add-padnoise-source-family-policy-instead-of-dense-break`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1194 (https://github.com/marang/riotbox/pull/1194)`
- Merge commit: `01b8bd441d5591ea327e3e8294b6ffa428be94eb`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1219-dense-smoke-1; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1219-matrix-smoke-1; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1219-source-wav-smoke-1; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1219-edge-smoke-review; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1219-suite-smoke-2; scripts/run_compact.sh /tmp/riotbox-1219-audio-qa-ci.log just audio-qa-ci; scripts/run_compact.sh /tmp/riotbox-1219-just-ci-final.log just ci; GitHub Actions rust-ci passed for PR #1194`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md; docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1220 bad-timing cautious arrangement policy and user cue proof`

## Why This Ticket Existed

Software: pad/noise edge sources no longer fall through the dense-break pressure rule when noisy high-band motion looks transient-like. Musician: pad/noise material is treated as a gated texture/cautious cue candidate instead of pretending to be a breakbeat.

## What Shipped

- Added a bounded pad_noise pressure policy, edge diagnostic expectations and negative mutation for missing pad/noise policy, professional-output suite key metrics for edge policy families/signals, and roadmap/benchmark/decision-log documentation.

## Notes

- None
