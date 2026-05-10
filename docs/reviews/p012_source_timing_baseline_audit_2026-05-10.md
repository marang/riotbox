# P012 Source Timing Baseline Audit - 2026-05-10

## Scope

Baseline audit for `RIOTBOX-713`, the first active slice under
`P012 | Source Timing Intelligence`.

Reviewed:

- `docs/plans/source_timing_intelligence_plan.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/phase_definition_of_done.md`
- `docs/execution_roadmap.md`
- `docs/benchmarks/source_timing_*`
- Source Graph timing contract and probe candidate code
- `riotbox-audio` source timing probe CLI and generated-audio smokes
- real local example WAV probe behavior from `data/test_audio/examples/`

This audit is a planning / baseline slice. It does not change timing behavior.

## Current Real Surface

The current P012 timing surface is already more than a placeholder:

- Source Graph has a replay-safe `TimingModel` with primary and alternate
  hypotheses, beat/bar/phrase grids, anchors, drift, groove residuals, timing
  quality, warnings, and degraded policy.
- The Rust probe path can analyze PCM WAV windows, extract onset times and
  strengths, score BPM candidates, score downbeat phases, build preliminary
  grids, classify anchors, summarize groove residuals, and emit a bounded JSON
  probe summary.
- Generated-audio smokes cover three important readiness cases: locked accented
  break, degraded silence, and ambiguous flat pulse.
- The shared Jam source timing summary feeds TUI, observer, and observer/audio
  correlation surfaces without introducing a second timing authority.
- Listening manifests and observer/audio summaries already validate compact
  source timing evidence, including anchor and groove evidence.

## Current Boundaries

The remaining gap is not "there is no beat detector". The gap is that real
short user loops still fail the musician-facing trust boundary in ways that are
too coarse:

- BPM and beat evidence are often stable on real drum loops.
- Downbeat evidence can be stable on several local examples.
- `requires_manual_confirm` still stays true because short-loop phrase evidence
  remains `not_enough_material` / `phrase_uncertain`.
- Melodic or pad-like examples correctly degrade to unavailable instead of
  pretending to be drum timing sources.

That boundary is appropriate for safety, but too blunt for the next product
step. Riotbox needs to distinguish "stable short loop, no long phrase evidence"
from "unsafe timing", so lanes can use short-loop grids conservatively while
still requiring confirmation for destructive or phrase-level moves.

## Real Example Probe Check

Command:

```bash
for source in data/test_audio/examples/*.wav; do
  cargo run -q -p riotbox-audio --bin source_timing_probe -- --json "$source"
done
```

Observed summary:

| Source | Cue | Readiness | Manual confirm | BPM | Beat | Downbeat | Phrase | Warnings |
| --- | --- | --- | --- | ---: | --- | --- | --- | --- |
| `Beat03_130BPM(Full).wav` | `needs confirm` | `ready` | yes | 130.285 | stable | stable | not_enough_material | phrase_uncertain |
| `Beat08_128BPM(Full).wav` | `needs confirm` | `ready` | yes | 128.397 | stable | stable | not_enough_material | phrase_uncertain |
| `Beat20_128BPM(Full).wav` | `needs confirm` | `weak` | yes | 128.397 | stable | weak | not_enough_material | phrase_uncertain, ambiguous_downbeat |
| `DH_BeatC_120-01.wav` | `needs confirm` | `ready` | yes | 120.185 | stable | stable | not_enough_material | phrase_uncertain |
| `DH_BeatC_KickSnr_120-01.wav` | `needs confirm` | `ready` | yes | 120.185 | stable | stable | not_enough_material | phrase_uncertain |
| `DH_Fadapad_120_A.wav` | `needs confirm` | `unavailable` | yes | none | unavailable | unavailable | unavailable | low_timing_confidence, weak_kick_anchor |
| `DH_RushArp_120_A.wav` | `needs confirm` | `unavailable` | yes | none | unavailable | unavailable | unavailable | low_timing_confidence, weak_kick_anchor |

Interpretation:

- the examples now show useful BPM/downbeat signal on several short drum loops
- the probe summary can look contradictory to a human because `readiness=ready`
  and `manual_confirm=yes` can appear together
- the next slice should refine readiness language and policy around short-loop
  phrase absence instead of loosening safety thresholds blindly

## Verification Run

The P012 baseline gates passed locally:

```bash
just source-timing-fixture-catalog-validator-fixtures
just source-timing-wav-probe
just source-timing-bpm-candidates
just source-timing-beat-evidence
just source-timing-downbeat-evidence
just source-timing-readiness-report
just source-timing-drift-report
just source-timing-phrase-grid
just source-timing-candidate-confidence-report
just generated-source-timing-probe-json-smoke
just generated-degraded-source-timing-probe-json-smoke
just generated-ambiguous-source-timing-probe-json-smoke
```

Important generated-audio outputs:

- locked generated break: BPM `128.397`, beat score `0.979`, downbeat score
  `0.565`, anchors `11`, groove residuals `4`
- degraded silence: cue `needs confirm`, warnings
  `low_timing_confidence,weak_kick_anchor`
- ambiguous flat pulse: BPM `128.397`, beat score `0.977`, downbeat score
  `0.268`, alternate downbeat phases `3`, anchors `16`

## Chosen Next Slice

Next implementation slice:

`RIOTBOX-714 - Refine short-loop source timing readiness semantics`

Goal:

- split stable short-loop grid evidence from long-phrase confidence
- make `readiness`, `requires_manual_confirm`, and cue language less
  contradictory for real short drum loops
- preserve conservative degradation for weak downbeat, ambiguous flat pulses,
  silence, melodic/pad examples, and destructive phrase-level actions
- add tests against generated short-loop families and at least one local
  example-style fixture pattern

This is the smallest useful P012 implementation slice because it improves the
musician-facing trust surface without creating a new analyzer architecture and
without prematurely relaxing lane behavior.

## Non-Goals For The Next Slice

- no Python runtime dependency
- no lane-local timing model
- no broad arbitrary-audio detector claim
- no forced use of original source audio in generated output
- no automatic destructive action unlock based only on BPM/beat evidence
