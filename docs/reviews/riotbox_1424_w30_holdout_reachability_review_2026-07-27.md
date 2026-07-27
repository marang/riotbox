# RIOTBOX-1424 W-30 Holdout Reachability Review

Date: 2026-07-27  
Phase: P023 / Controlled Expansion  
Classification: `contract_enabler`  
Directly enabled audible follow-up: RIOTBOX-1422 fresh W-30 Hard holdout

## Outcome

The existing `w30_live_path_render` now supports a bounded non-listening
reachability gate:

```bash
cargo run -p riotbox-app --bin w30_live_path_render -- \
  --source path/to/source.wav \
  --output /tmp/w30-preflight \
  --bpm 130 \
  --include-resample \
  --preflight-only
```

`--preflight-only` emits `w30-reachability-preflight.json`, returns nonzero for
an ineligible case, and stops before candidate review WAV generation.
`--require-exact-hit-shaper` applies the same gate and continues through the
existing renderer only when the report permits candidate generation. Ordinary
`--include-resample` behavior remains available for texture and non-hit-shaper
diagnostics.

The report records:

- Rust timing readiness, grid-use policy, requested BPM/downbeat route, primary
  BPM delta, and the unchanged shared `1 BPM` confirmation tolerance
- the product Source Graph primary BPM and grid-use result
- policy and recipe selected by the actual Source Graph -> Session -> queue ->
  commit -> capture promotion -> internal resample -> Hard projection path
- exact hit-shaper applicability, evaluation, and calibration
- explicit blockers and eligibility for later candidate-WAV generation

The exact applicability predicate is owned by `W30ResampleTapState` and reused
by projection calibration and preflight. The diagnostic does not contain a
second Hard selector.

BPM-only confirmation retains the shared analyzer-primary match rule. A paired
independently known BPM and downbeat instead verifies that the product installed
the existing typed manual hypothesis; it does not need to match an analyzer
tempo alias.

## Regression Evidence

Consumed development cases reproduce the failures that blocked H14-H16:

| Case | Timing result | Product result | Candidate permission |
| --- | --- | --- | --- |
| H14 tense bass | `140` requested / `139.75156` primary | transient policy, recipe unavailable | rejected |
| H15 Psychic | `190` requested / `141.50945` primary | not evaluated | rejected |
| H16 melodic skippy | `140` requested / `154.63918` primary | not evaluated | rejected |
| consumed snare one-shot | primary grid unavailable | not evaluated | rejected |
| Beat03 development | `130` requested / `130.28494` primary | `source_hit_shaper_v3`, exact calibration evaluated and successful | allowed |

The gated Beat03 run then completed all inherited render and directional gates.
The rejected H14 normal-mode run wrote no numbered candidate review WAV.

An explicit Beat03 downbeat of `0` made the projected capture unavailable,
while the same source with only a matching BPM reached the exact recipe. This
is retained as evidence that preflight must report, not invent, phase.

## Holdout Contract

`source_holdout_rotation_v1.json` now requires a non-listening W-30 reachability
screen after candidate freeze and before holdout candidate review WAVs. A
qualified set still needs at least two sources across two families, with at
least one case selecting and successfully calibrating the exact hit-shaper
path. Non-selection and timing mismatch are ineligible rather than passing
alternates.

No current fresh Holdout A or B source was opened, timing-probed, rendered, or
auditioned during RIOTBOX-1424. Commercial reference material was not used.

## Verification

- `cargo test -p riotbox-app --bin w30_live_path_render`
- `cargo test -p riotbox-audio w30`
- `cargo check -p riotbox-app --bin w30_live_path_render`
- `just source-holdout-rotation-fixtures`
- consumed-case H14/H15/H16/one-shot preflight runs
- consumed Beat03 preflight-only and gated full-render runs

`human_verdict: unverified` is intentional. This issue changes reachability and
QA ownership, not audible DSP; listening belongs to the subsequently frozen
RIOTBOX-1422 candidate.
