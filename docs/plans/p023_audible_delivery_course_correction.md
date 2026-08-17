# P023 Audible Delivery Course Correction

Status: accepted operating plan
Linear: RIOTBOX-1439
Direct audible follow-ups: RIOTBOX-1417, RIOTBOX-1440

## Purpose

Restore a fast musician-value loop without weakening Riotbox's source,
Holdout, realtime, replay, exact-output, or release gates. Recent fail-closed
work proved that correct contracts can still be applied in the wrong order:
substantial product and qualification machinery was built before the proposed
musical value had earned a provisional human keep.

This plan separates reversible Development discovery from immutable product
qualification. Both stages remain inside one Linear-first audible slice; the
first stage is not a separate product-completion claim.

## Decision Table

| State | Required work | Exit | Prohibited claim |
| --- | --- | --- | --- |
| `development_exploration` | one exact registered Development source, existing render seam where possible, technical safety/artifact preflight, at most three materially distinct variants, early bounded human usefulness check | provisional keep, explicit stop, or three-variant limit | product behavior, source-general quality, demo/release readiness, hardness, Holdout authorization |
| `stop` | remove temporary experiment behavior, retain only evidence needed to prevent repetition | close or reshape the audible ticket | tuning the same weak mechanism past its limit |
| `freeze_for_qualification` | select one kept mechanism, write its versioned contract and durable Decision, rebuild the candidate from that frozen contract | source-blind implementation begins | treating the exploratory artifact or verdict as qualification evidence |
| `product_qualification` | product-spine integration, applicable source-diversity matrix, replay/callback/exact-output proof, formal structured listening, PR/CI | merge or fail closed | changing frozen mappings, constants, or thresholds from qualification/holdout results |

## Development Exploration

Exploration answers one narrow question: is the intended musical role obvious,
useful, and worth performing?

- Start with one exact path/hash already registered for Development. Do not
  discover source directories or open Holdout/commercial-reference audio.
- Prefer existing actions and exact RuntimeMix seams. Temporary experiment code
  is allowed only when needed to hear the causal idea; it remains explicitly
  non-product and must be removed if the idea stops.
- Change causal topology between variants, not merely a tiny scalar after
  near-identity feedback. Count all variants of the same causal mechanism
  against the limit of three.
- Before playback, apply the normal exact-artifact preflight and listening
  safety workflow. The listener may say whether the result is useful, obvious,
  worth triggering, or directionally wrong.
- Keep exploratory notes lightweight and local. Do not create a Decision-Log
  entry, new validator framework, source matrix, release pack, or formal
  product verdict for reversible operations.

Stop immediately when the result is near-identical for its intended role, has
the wrong audible owner, loses the hook/source, is not worth triggering, or
reaches three failed variants. A stopped exploration may preserve one concise
durable negative record when otherwise the same mechanism is likely to be
reopened.

## Promotion Boundary

A provisional keep authorizes qualification work; it does not satisfy it.
Before product implementation:

1. select exactly one mechanism and version;
2. freeze its mappings, constants, controls, source partition, stopping rule,
   and claimed audible role;
3. record the durable Decision;
4. rebuild the exact candidate from the frozen contract;
5. implement through Source Graph, Session/replay, Action Lexicon, queue/commit,
   observer, and RuntimeMix surfaces that the product claim actually needs;
6. run the applicable source-diversity and formal human-review gates.

Qualification and Holdout results may reject but may not tune the frozen
version. A required change creates a new version and Decision.

## Work And Command Budget

- Exploration: focused tests for touched seams plus one exact render/preflight;
  do not run the broad phase suite after every variant.
- Qualification: focused product-spine, source-matrix, callback, replay, and
  listening gates selected by the frozen claim.
- PR/merge: branch review and the broad gate required by the workflow; `just
  ci` remains the current final default until RIOTBOX-1399 lands a measured
  scoped replacement.

Generated `target/` and `artifacts/` trees are reusable local state. Never
delete them automatically. RIOTBOX-1399 owns measured command/runtime and safe
retention improvements when those costs block the audible path.

## First Application

RIOTBOX-1417 applied this order to one source-derived MC-202 instigator role.
Three materially distinct Development variants produced no provisional human
keep, so temporary behavior was removed and the work stopped before frozen
qualification. The negative result is preserved in
`docs/reviews/riotbox_1417_mc202_realized_role_development_rejection_2026-08-16.md`.
Any successor must be a new Linear-first audible slice with a materially
different musical owner or grammar, not more scalar tuning of the rejected
retrigger or full-bed-cut mechanisms.

RIOTBOX-1440 then applied the keep path successfully. One source-recognizable
W-30 sampler turnaround earned an early Development keep, was frozen before
product work, rebuilt as the explicit `w30.hook_turnaround` action without
retuning, and passed dense, tonal, and sparse Development qualification through
the existing Action/Session/replay/RuntimeMix spine. The one formal product A/B
also passed: the effect remained useful, source-recognizable, cleanly returning,
and worth live triggering. This validates the course-correction order while
remaining a bounded W-30 articulation result, not a hardness, Golden Path, or
release-readiness claim.
