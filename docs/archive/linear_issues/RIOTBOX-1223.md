# `RIOTBOX-1223` Improve pad/noise gated texture path beyond weak routing

- Ticket: `RIOTBOX-1223`
- Title: `Improve pad/noise gated texture path beyond weak routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1223/improve-padnoise-gated-texture-path-beyond-weak-routing`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1223-improve-padnoise-gated-texture-path-beyond-weak-routing`
- Linear branch: `feature/riotbox-1223-improve-padnoise-gated-texture-path-beyond-weak-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1201 (https://github.com/marang/riotbox/pull/1201)`
- Merge commit: `b6f0e28a9b19cefb0d3639d664224e848f1d2a55`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py`; `git diff --check`; `just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1223-edge-smoke-3`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1223-suite-smoke-2`; `just ci`; `just audio-qa-ci`; `GitHub Actions rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `Future pad/noise work should improve source selection, UI cues, and listening-review labels before treating pad/noise output as demo-ready`

## Why This Ticket Existed

P022 already prevented pad/noise material from being misclassified as dense-break proof, but that path was still mostly weak routing. Riotbox needed a bounded audible gated texture/stab behavior with proof that the material was transformed without becoming a false quality claim.

## What Shipped

- Added source-derived pad_noise_texture_policy with gate/stab offsets, gate duty, texture gain, stab gain, candidate count, and fixed-choice distance.
- Rendered the pad/noise texture layer into hook/chop/pressure/restore buses only for pad_noise sources.
- Surfaced pad/noise texture proof in dense reports, edge-source diagnostics, and professional-output suite key metrics.
- Added positive gates plus negative mutations for non-source-derived and fixed-collapsed pad/noise texture reports.
- Updated P022 benchmark docs, roadmap text, and RBX-077 while keeping quality_proof false and human_verdict unverified.

## Notes

- Final suite key metrics reported 48 pad/noise texture candidates, 44100-frame gate fixed distance, 352800-frame stab fixed distance, 352800-frame gate/stab distance, and 3.38 transient ratio.
