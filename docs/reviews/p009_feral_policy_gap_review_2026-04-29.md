# P009 Feral Policy Gap Review 2026-04-29

## Scope

This review checks `P009 | Feral Policy Layer` after the first bounded Feral
consumers landed.

Reviewed:

- `docs/phase_definition_of_done.md`
- `docs/specs/source_graph_spec.md`
- `docs/specs/action_lexicon_spec.md`
- `docs/specs/audio_qa_workflow_spec.md`
- `docs/jam_recipes.md`
- current Feral scorecard, W-30, TR-909, MC-202, UI, and audio QA tests

## Current Status

P009 is materially active and useful, but not exit-clean yet.

The architecture boundary is holding:

- Feral behavior remains a policy / projection layer.
- No second Source Graph, sampler, arranger, or Ghost path was introduced.
- Existing Action Lexicon, queue / commit, render-state, and UI contracts still
  own the live behavior.

## Satisfied Or Mostly Satisfied

| Criterion | Status | Evidence |
| --- | --- | --- |
| Feral scorecard exists | Satisfied | `FeralScorecardView` exposes readiness, break potential, hook count, support count, quote risk, capture count, reason, and warnings. |
| Scorecard has a visible consumer | Satisfied | Source/Jam surfaces show `feral ready` and `feral needs support`; Recipe 12 teaches the path. |
| W-30 consumes Feral evidence | Satisfied for bounded MVP | `w30.browse_slice_pool` can prefer a supported Feral capture and proves the source-window preview differs from the cyclic control preview. |
| TR-909 consumes Feral evidence | Satisfied for bounded MVP | Source support can lift `steady_pulse` to `break_lift` with `feral break lift` diagnostics and offline output delta proof. |
| MC-202 consumes Feral evidence | Satisfied for bounded MVP | Feral break support can choose `answer_space` and sparse note budget with offline output delta proof. |
| Feral remains replay-safe | Satisfied for current consumers | Consumers use existing session, action, queue, commit, and render projection seams. |
| UI teaches current Feral path | Mostly satisfied | Recipe 12 and Jam suggested gestures explain `feral ready`; diagnostics expose `feral break lift` and W-30 feral slice-pool reason. |

## Still Weak

- P009 does not yet have a single current exit review tying scorecard,
  consumer selection, audio proof, and musician-facing workflow together.
- The current `feral_grid_pack` proves a useful generated listening pack, but
  its manifest / summary is not yet explicitly tied to the live
  `FeralScorecardView` explanation.
- W-30, TR-909, and MC-202 each have bounded Feral consumers, but Riotbox still
  lacks one coherent "why this run is feral" artifact that a musician or tester
  can read beside the WAVs.
- Abuse-mix / rebake / promotion policy is still mostly represented by W-30
  resample and reuse seams, not by a Feral-specific promotion decision.
- Quote-risk handling is visible in the scorecard, but it is not yet a guardrail
  that affects candidate choice or generated pack reporting.

## Current Evidence References

- Scorecard model and readiness:
  `crates/riotbox-core/src/view/jam/capture_actions.rs`
- Scorecard tests:
  `crates/riotbox-core/src/view/jam/tests/feral_scorecard_tests.rs`
- W-30 Feral slice-pool target and preview delta:
  `crates/riotbox-app/src/jam_app/tests/w30_queue_core.rs`
- Cross-lane Feral consumer consistency:
  `crates/riotbox-app/src/jam_app/tests/feral_support_runtime_controls.rs`
- TR-909 Feral source-support output proof:
  `crates/riotbox-app/src/jam_app/tests/tr909_takeover_source_support.rs`
- Feral recipe:
  `docs/jam_recipes.md`
- Generated listening-pack and observer/audio QA:
  `just audio-qa-ci`

## Recommended Next Slice

`RIOTBOX-401`: add a machine-readable Feral scorecard summary to the generated
Feral grid pack manifest / pack summary.

Minimum scope:

- derive a compact scorecard-like explanation for the generated Feral grid pack
- include why the pack is considered feral, which lane gestures contributed,
  and whether the result is source-backed, generated, or fallback-like
- validate the new manifest fields with focused tests
- keep the pack as an offline QA artifact, not a new runtime behavior path

Why this next:

- it connects the current scorecard policy to the strongest existing audio QA
  artifact
- it gives musicians and testers an explanation next to the WAVs
- it avoids adding more live behavior before P009 evidence is easier to inspect

## Not Recommended Yet

- Do not start Ghost Feral autonomy.
- Do not add a second break-rebuild engine.
- Do not add broad quote-risk enforcement before the scorecard is attached to
  generated Feral QA artifacts.
- Do not close P009 until the generated Feral run explains itself and the
  remaining rebake / promotion gap is either implemented or explicitly scoped
  out of MVP.
