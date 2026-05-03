# Riotbox Source Timing Intelligence

## Summary

Riotbox needs an all-lane timing foundation, not just beat detection. This slice introduces **Source Timing Intelligence** as a Rust-first, replay-safe Source Graph contract: BPM, beat, downbeat, bar, phrase, drift, groove/microtiming, anchor confidence, and degraded modes become shared timing surfaces for TR/Kick-Bass, MC-202, W-30, Scene Brain, Feral policy, and Ghost.

Defaults:

- Rust-first product path; no durable Python runtime dependency.
- External MIR/Python tools may be used only for research comparison, never as the runtime contract.
- TR/Kick-Bass, MC-202, and W-30 are all required first-class consumers, not alternatives.
- Every timing improvement must prove both control path and output path: Graph/state/action plus WAV artifacts, metrics, and listening evidence.

## Key Changes

- Source Graph becomes the shared musical timing contract:
  - multiple `TimingHypothesis` candidates instead of one BPM value
  - primary hypothesis plus half-time, double-time, and ambiguous alternatives
  - beat, downbeat, bar, and phrase grids with confidence
  - source anchors: kick, snare/backbeat, fills, loop windows, answer slots, capture candidates
  - drift report over 4/8/16/32 bars
  - groove/microtiming residuals so Riotbox recognizes feel instead of flattening everything to a hard grid
  - timing warnings such as `weak_kick_anchor`, `ambiguous_downbeat`, `double_time_possible`, and `drift_high`
- Audio Core remains the realtime authority:
  - analysis never runs inside the audio callback
  - Source Graph provides musical alignment data
  - scheduler/transport still apply commit boundaries deterministically
  - lanes consume only bounded serialized timing data
- "TR-202" is treated in this plan as a low-end/kick-bass consumer of the existing TR/Bass policy context, not as a separate shadow architecture. If a dedicated TR-202 lane lands later, it must use the same timing contract.

## Implementation Changes

- **Spec + Contract**
  - Add `docs/specs/source_timing_intelligence_spec.md`.
  - Update Source Graph, Audio Core, Validation/Benchmark, Fixture Corpus, and Audio QA workflow specs.
  - Add a Decision Log entry: Rust-first Timing Intelligence; optional external bakeoff is research only.
- **Core Model**
  - Extend `TimingModel` with hypotheses, primary selection, drift, microtiming, timing quality, and warnings.
  - Add helpers for primary grid lookup, nearest anchor, source-time-to-bar/phrase, confidence classification, and degraded timing policy.
  - Persist the selected timing hypothesis and Source Graph hash so restore/reanalysis cannot silently change musical alignment.
- **Rust Analyzer Spike**
  - Build a deterministic WAV-based analysis CLI/library:
    - onset envelope
    - low-band/kick candidate extraction
    - snare/backbeat candidate extraction
    - BPM/phase hypothesis generation
    - downbeat scoring
    - bar/phrase grid proposal
    - drift report
  - Output is a Source Graph timing payload, not a provider-specific blob.
- **All-Lane Proof**
  - TR/Kick-Bass: kick reinforcement, bass/sub attack, short/hard hits, sparse low-end, and downbeat support align to detected source timing and degrade conservatively on low confidence.
  - MC-202: question/answer phrases use bar/phrase slots from the graph, not hardcoded example positions.
  - W-30: chops, loop windows, capture ranges, and pad candidates come from transient + grid + confidence, including loop-closure/drift checks.
  - Scene Brain and Ghost may read timing only through Source Graph/view models, not by inventing separate timing logic.
- **Musician UX**
  - Jam/Source surfaces show BPM, beat, bar, phrase, timing quality, and warnings.
  - Low-confidence behavior must be audible and visible: more conservative lane suggestions, clear degraded status, and no false "locked" feel.
  - Each slice provides an executable listening proof with source, Riotbox output, and source-then-Riotbox A/B.

## Quality And Test Plan

- **Timing Metrics**
  - BPM error against synthetic/annotated fixtures
  - beat hit rate within 35 ms and 70 ms
  - downbeat accuracy
  - phrase boundary agreement within expected ranges
  - drift after 4/8/16/32 bars
  - confidence calibration: weak fixtures must not report high confidence
- **Audio Metrics**
  - generated-lane drift vs source grid
  - transient/source alignment for kick, bass, MC-202, and W-30 events
  - kick-bass attack offset when relevant
  - peak, RMS, low-band RMS, crest factor
  - onset density before/after
  - silence/fallback-collapse checks
  - source-vs-control comparisons so source-backed features cannot silently collapse to fallback
- **Musical Quality Checks**
  - The contract does not enforce one bass aesthetic.
  - Bass, kick, and low-end may be short, hard, long, deep, thin, broken, dry, dubby, aggressive, restrained, or absent.
  - QA checks whether timing, intent, level, drift, transients, and collapse risks are controlled, not whether output is "fat enough."
  - Each lane must show that its output matches the selected scene/lane policy and does not accidentally drift, clip, phase, or collapse to fallback.
- **Fixture Scenarios**
  - clean 128 BPM rhythm: high-confidence full-lane lock
  - dense break-heavy material: ghost notes must not destroy downbeat selection
  - hook-forward material: answer/capture candidates without quote-risk blindness
  - half-time/double-time ambiguity: multiple hypotheses remain visible
  - weak timing: degraded mode instead of false confidence
  - drift/live-feel material: drift report and groove residuals remain musically usable
- **Acceptance Gate**
  - Every lane proof must pass both control path and output path.
  - Artifacts live under `artifacts/audio_qa/...` with source, output, A/B, manifest, and metrics.
  - Golden renders cover TR/Kick-Bass, MC-202 QA, and W-30 chops.
  - Observer/audio correlation proves that action, state, render status, and audio output agree.

## Assumptions

- The plan intentionally does not reduce the first timing program to a single minimal consumer; all three main musical consumers are included.
- Work is still split into reviewable slices: spec, core contract, analyzer, QA harness, then the three lane proofs.
- The goal is not one fixed sound aesthetic, but provably good audio output: rhythmically precise, musically intentional, dynamically controlled, and honestly degraded when confidence is weak.
