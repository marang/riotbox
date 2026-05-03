# P009 Feral Policy Exit Review 2026-04-29

## Scope

This review refreshes `docs/reviews/p009_feral_policy_gap_review_2026-04-29.md`
after RIOTBOX-401 through RIOTBOX-404.

Reviewed evidence:

- Feral scorecard model, UI surfaces, and source graph readiness tests.
- Generated Feral grid pack manifest and validator coverage.
- W-30 / TR-909 / MC-202 bounded Feral consumers.
- W-30 resample / rebake action result and quote-risk hold behavior.
- Audio QA workflow and listening-manifest contract docs.

## Decision

P009 is MVP-exit-clean for the bounded Feral policy layer.

This does not mean Riotbox has a full autonomous break-rebuild arranger. It
means the MVP policy layer now has enough explicit scorecard, consumer, audio
QA, resample reuse, and quote-risk guard evidence to move the main roadmap
forward without adding a shadow Feral architecture.

## Done Criteria Check

| Criterion | Status | Evidence |
| --- | --- | --- |
| Harvest produces usable fragment candidates | Satisfied for MVP | `FeralScorecardView` reads hook fragments, capture candidates, break-support relationships, and quote-risk relationships from Source Graph evidence. |
| At least one break rebuild path is musically interesting | Satisfied for bounded MVP | W-30 Feral slice-pool selection, TR-909 `feral break lift`, MC-202 hook-response restraint, and the generated Feral grid pack all exercise a cross-lane Feral path. |
| Hook-fragment handling exists without full-quote dependence | Satisfied for MVP | Scorecard warnings expose quote risk, generated Feral manifests record `quote_risk_count`, and W-30 Feral rebake approval is held when quote-risk evidence exists. |
| Resample reuse is real, not decorative | Satisfied | W-30 internal resample writes a reusable bus-print artifact, records lineage/depth, updates the action result, and projects capture notes back into Jam view. |
| Feral scorecard metrics can be generated | Satisfied | Scorecard is present in live Source/Jam view and in generated Feral grid listening-pack manifests. |

## Gap Review Closure

The previous gap review found five weak spots.

- Exit review missing: closed by this document.
- Generated Feral grid pack did not explain why it was Feral: closed by the
  `feral_scorecard` block in the generated pack manifest.
- QA artifact lacked validator coverage: closed by optional `feral_scorecard`
  validation in `scripts/validate_listening_manifest_json.py` and generated-pack
  manifest validation.
- Rebake / promotion policy was implicit: closed for MVP by the W-30
  `promote.resample` Feral rebake action result and capture note.
- Quote-risk was visible but not a guardrail: closed for the bounded rebake cue
  by holding approval when high quote-risk relationships are present.

## Current Evidence References

- Scorecard model and readiness:
  `crates/riotbox-core/src/view/jam/capture_actions.rs`
- Scorecard tests:
  `crates/riotbox-core/src/view/jam/tests/feral_scorecard_tests.rs`
- Generated Feral grid scorecard:
  `crates/riotbox-audio/src/bin/feral_grid_pack/render_stems.rs`
- Generated scorecard validator:
  `scripts/validate_listening_manifest_json.py`
- Listening manifest scorecard contract:
  `docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md`
- W-30 Feral rebake approval / quote-risk hold:
  `crates/riotbox-app/src/jam_app/commit.rs`
- W-30 Feral rebake tests:
  `crates/riotbox-app/src/jam_app/tests/w30_committed_preview_resample.rs`
  and `crates/riotbox-app/src/jam_app/tests/w30_feral_rebake_policy.rs`
- Feral grid and observer/audio QA:
  `just audio-qa-ci`

## Not Claimed

- No broad quote-risk enforcement across every action.
- No Ghost Feral autonomy.
- No second arranger, sampler, Source Graph, or Feral-specific runtime path.
- No claim that generated Feral packs replace human listening.

## Recommended Next Step

Move the main implementation lane out of P009 and back to the next roadmap
slice that improves the musician-facing playable loop.

If a future feature makes new autonomous promotion decisions, it should reuse
the existing scorecard, lineage, quote-risk, action/result, and audio-QA seams
instead of reopening P009 as a separate architecture.
