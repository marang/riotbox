# P012 Post Source Transport Spine Review - 2026-05-26

## Scope

Focused current-state `review-codebase` pass after the `RIOTBOX-967` source
transport / adaptive Source Map / monitor / capture parent closed and
`RIOTBOX-993` made grid-use fixture phrase counts explicit.

Reviewed:

- `docs/execution_roadmap.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/reviews/p012_real_source_timing_confidence_review_2026-05-22.md`
- `Justfile`
- `scripts/write_p012_all_lane_proof_summary.py`
- `scripts/validate_source_timing_grid_use_contract_fixtures.py`
- `crates/riotbox-core/src/view/jam/source_timing_summary.rs`
- `crates/riotbox-app/src/ui/source_trust_summary.rs`
- `crates/riotbox-app/src/bin/observer_audio_correlate/*source_timing*`
- current local example report from
  `just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md`

This review did not change analyzer, Session, action, JamAppState, UI,
observer schema, realtime audio, or audio-output behavior.

## Summary

The current P012 source-timing spine is coherent after the source-transport
closeout. The roadmap still points at P012 as the immediate execution track
(`docs/execution_roadmap.md:577`), and the near-next direction is still
real-source timing confidence, shared Jam / Source / observer language, concrete
lane-output proof, and honest fallback/manual-confirm behavior
(`docs/execution_roadmap.md:591`).

The old review follow-ups around downbeat ambiguity visibility, Recipe 15
strictness, compact P012 summary detail, observer/audio phrase evidence, and
grid-use fixture phrase-count ownership should not be reopened as duplicate
tickets. Current `main` already exposes those surfaces:

- `SourceTimingSummaryView` owns downbeat offset, score, score gap, alternate
  phase count, beat/bar/phrase counts, and anchor/groove evidence
  (`crates/riotbox-core/src/view/jam/source_timing_summary.rs:9`,
  `crates/riotbox-core/src/view/jam/source_timing_summary.rs:172`).
- Jam / Help source timing lines render the phase chip and ambiguity detail
  through the shared summary (`crates/riotbox-app/src/ui/source_trust_summary.rs:194`,
  `crates/riotbox-app/src/ui/source_trust_summary.rs:308`).
- Observer/audio summaries preserve ambiguous downbeat evidence and phrase
  evidence (`crates/riotbox-app/src/bin/observer_audio_correlate/source_timing_evidence_tests.rs:47`).
- The phase-level gate uses strict Recipe 15 fixture requirements
  (`Justfile:249`, `Justfile:512`).
- The compact P012 proof summary includes generated Feral-grid observer/audio
  cue, actionability, grid source, observer counts, phrase counts, grid
  compatibility, downbeat ambiguity, anchor/groove alignment, and output issues
  (`scripts/write_p012_all_lane_proof_summary.py:111`).

## Findings

### Minor - Grid-use fixture downbeat evidence is still helper-derived

- Location: `scripts/validate_source_timing_grid_use_contract_fixtures.py:50`
- Location: `scripts/validate_source_timing_grid_use_contract_fixtures.py:289`
- Category: scope
- Severity: minor

`GridUseCase` now owns beat/bar counts and phrase count/bar-count evidence
explicitly, which makes those expectations reviewable in the case table. The
same fixture still derives `primary_downbeat_score`, `primary_downbeat_margin`,
and `alternate_downbeat_phase_count` inside `apply_timing_fields(...)` from only
`downbeat_status`.

Impact: current generated cases are valid, but future Beat20-like ambiguity
variants could hide score, margin, or alternate-phase assumptions in helper
branching instead of in the case row reviewers read first.

Suggestion: add explicit downbeat-evidence fields to `GridUseCase` and copy them
through `apply_timing_fields(...)`, keeping generated fixture semantics
unchanged.

## Non-Findings

- The P012 proof gate is not silently skipping Recipe 15 fixtures at phase level;
  it uses `recipe15-feral-grid-auto-proof-strict`.
- The compact proof summary is no longer just a pass/fail line for generated
  observer/audio paths.
- The real-source example report currently distinguishes stable short-loop
  manual-confirm rows from the Beat20 ambiguous-downbeat row. In the current
  report, Beat20 remains `manual_confirm_only` with stable beat evidence,
  ambiguous downbeat, margin `0.005`, three downbeat alternates, and
  transient-only anchors (`/tmp/riotbox-next-source-timing-report.md:9`).
- No second timing authority, Source Map truth, Session/replay truth, or
  app-local timing contract was found in the reviewed scope.

## Recommended Next Slice

Create one bounded P012 cleanup ticket:

`Make grid-use fixture downbeat evidence explicit`

Functional impact:

- no direct sound or UI change
- tighter proof harness reviewability
- future ambiguous-downbeat fixture variants can state score, margin, and
  alternate-phase evidence directly in the case table

After that small harness cleanup, the next product-facing P012 direction should
return to real-source timing confidence: use the current Beat20-like row as the
bounded target, improve only if a musically defensible anchor emerges, and keep
manual-confirm fallback intact for sources without kick/backbeat support.

## Verification

```bash
scripts/run_compact.sh /tmp/riotbox-next-source-timing-report.log just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md
git diff --check
```
