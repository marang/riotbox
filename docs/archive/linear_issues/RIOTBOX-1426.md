# `RIOTBOX-1426` P023: Derive a source-backed downbeat phase for externally supplied tempo

- Ticket: `RIOTBOX-1426`
- Title: `P023: Derive a source-backed downbeat phase for externally supplied tempo`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1426/p023-derive-a-source-backed-downbeat-phase-for-externally-supplied`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4`
- Status: `Done`
- Created: `2026-07-27`
- Started: `2026-07-27`
- Finished: `2026-07-27`
- Branch: `feature/riotbox-1426-p023-derive-source-backed-downbeat-phase-for-external-tempo`
- Linear branch: `feature/riotbox-1426-p023-derive-source-backed-downbeat-phase-for-external-tempo`
- Assignee: `Owner`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1380 (https://github.com/marang/riotbox/pull/1380)`
- Merge commit: `760fec3b896d913860af190f13988779b001e90d`
- Deleted from Linear: `2026-07-27`
- Verification: `focused Core/app regressions; observer and W-30 preflight regressions; seven-case/five-family local development matrix; cargo fmt; cargo test; cargo clippy; final just ci; GitHub Rust CI`
- Docs touched: `docs/benchmarks/w30_tempo_guided_development_v1.json; docs/engineering/audio_numeric_values.md; docs/specs/source_timing_intelligence_spec.md; docs/specs/source_graph_spec.md; docs/specs/fixture_corpus_spec.md; docs/research_decision_log.md; docs/reviews/riotbox_1426_tempo_guided_source_phase_review_2026-07-27.md`
- Follow-ups: `RIOTBOX-1422: freeze the merged timing mechanism and consume the restored reserve exactly once as H24; request structured listening only after timing, exact Hard recipe, callback calibration, and technical gates pass.`

## Why This Ticket Existed

H23 proved that independent provider/project tempo could disagree with the Rust
primary while BPM alone still could not justify downbeat phase zero. Riotbox
needed one bounded source-backed phase route before another fresh exact-path
attempt.

## What Shipped

- Added the typed `TempoGuided` Source Graph hypothesis: external BPM plus
  source-derived phase from real onset evidence.
- Required complete-bar support, score margin, and bounded drift; preserved
  analyzer evidence and explicit musician-manual timing.
- Persisted/restored the locked product grid without fabricating a
  `source_timing.confirm_grid` action.
- Exposed typed observer and W-30 preflight evidence and froze a seven-case,
  five-family development matrix.

## Notes

- Contract enabler only; no W-30 DSP or hit-shaper gate changed.
- No candidate WAV or human musical-quality verdict exists.
- Lucid Trigger, NES Chopin, Cave, Sector, and the untouched bad-timing reserve
  remained inaccessible until this merge.
