# `RIOTBOX-1215` Add pad/noise and bad-timing professional-output diagnostics

- Ticket: `RIOTBOX-1215`
- Title: `Add pad/noise and bad-timing professional-output diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1215/add-padnoise-and-bad-timing-professional-output-diagnostics`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1215-add-padnoise-and-bad-timing-professional-output-diagnostics`
- Linear branch: `feature/riotbox-1215-add-padnoise-and-bad-timing-professional-output-diagnostics`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1190 (https://github.com/marang/riotbox/pull/1190)`
- Merge commit: `fac208d89f2459a753d914a49dc0e6a1a2b41fbc`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py scripts/route_weak_output_fixes.py; just sound-excellence-source-corpus-fixtures; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1215-edge-source-smoke-final; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1215-professional-output-suite-final; scripts/run_compact.sh /tmp/riotbox-1215-audio-qa-ci.log just audio-qa-ci; scripts/run_compact.sh /tmp/riotbox-1215-just-ci-final.log just ci; post-review py_compile + edge-source smoke; GitHub rust-ci passed on PR #1190`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md; docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/benchmarks/sound_excellence_source_corpus_v1.json; docs/benchmarks/sound_excellence_source_corpus_v1_2026-06-05.md; docs/execution_roadmap.md`
- Follow-ups: `RIOTBOX-1216, RIOTBOX-1217`

## Why This Ticket Existed

P022/P023 sound-product coverage named pad/noise and bad-timing sources, but professional-output diagnostics only covered dense breaks, tonal hooks, and sparse bass pressure.

## What Shipped

- Added edge-source professional diagnostics for pad_noise_fadapad_120 and bad_timing_beat20_128, including source-timing reports, rendered WAV hashes, metrics, pressure-lift policy metadata, weak-output signals, and fix routing. Wired the report into audio-qa-ci and professional-output-suite as the 8th child, with mutation checks for silence, identical output, fallback-collapse correlation, and missing source-family metadata.

## Notes

- Diagnostic only: human_verdict remains unverified and quality_proof remains false. Bad-timing corpus now uses Beat20 downbeat ambiguity; Fadapad remains pad/noise material.
