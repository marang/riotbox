# RIOTBOX-1433 Audible Algorithm Value Audit — 2026-08-12

Work class: `contract_enabler`

Directly enabled audible follow-up: `RIOTBOX-1432`

## Outcome

The smallest high-value replacement target is the W-30 Golden Path's source
window / hook selection. Riotbox already renders real committed source PCM,
preserves it through Session and replay, and has a human-passed exact live
performance path. The weak link is earlier: the supported Recipe 17 captures
the transport-selected one-bar window, while the decoded Source Graph always
publishes its opening two bars as the sole loop candidate and publishes zero
hook candidates. No source evidence currently chooses the most memorable bar
or subwindow for the product hook.

The source-scored eight-slice policy in
`crates/riotbox-audio/src/bin/feral_grid_pack/w30_slice_choice.rs` does not
close this gap. It is bin-local diagnostic scaffolding, uses equal-width
RMS/delta/peak scoring, and never owns the Session, queue/commit, replay, or
exact live RuntimeMix decision.

Therefore RIOTBOX-1432 should replace the product selection seam, not promote
the diagnostic heuristic unchanged and not add another general QA framework.

## Evidence Map

| Algorithm family | Owner and musician consequence | Source evidence and current proof | Status / principal risk | Proportionality judgment |
| --- | --- | --- | --- | --- |
| PCM decode and source cache | Sidecar ingest plus `riotbox-audio::source_audio`; loading a WAV makes real source frames available to capture and playback | File bytes, WAV format, hashes, source identity; unit/integration coverage and exact live playback | Product foundation; format refusal can make material unavailable | **Retain.** Simple, audible, and necessary. |
| Source Timing and manual confirmation | Rust timing probe, Source Graph timing model, and committed confirmation; capture, transport, all lanes, replay, and scenes share one grid | Real PCM onset evidence plus explicit musician BPM/downbeat when required; multi-source regressions and exact live-path proof | Product algorithm; arbitrary-source confidence remains incomplete | **Retain.** Use RIOTBOX-1033/1042 only when a concrete source blocks RIOTBOX-1432; do not broaden timing work first. |
| Sidecar sections, loop candidate, and coarse features | `python/sidecar/json_stdio_sidecar.py`; supplies source map buckets, two coarse energy sections, one opening loop candidate, and phrase features | Real PCM energy and simple proxies, but the loop is always the opening window, sections split the file coarsely, and `hook_candidate_count` is zero | Near-production scaffold; fixed heuristic can look source-aware while leaving hook choice feature-independent | **Replace only the hook-selection consequence in RIOTBOX-1432.** Keep useful decode/features; do not claim the current loop candidate is musical intelligence. |
| W-30 capture, pad playback, damage, and recall | `capture_helpers`, W-30 side effects, runtime preview renderer, Session/replay; `capture -> audition -> promote -> w` produces the playable hook | Exact captured frames and lineage drive output; same-source replay, cross-source diversity, callback-path proof, and human-passed normal/damaged Beat03 elements | Product algorithm; selection quality is the gap, not PCM playback | **Retain renderer and product spine. Replace only selection.** |
| Offline W-30 slice choice and Feral pack policies | `feral_grid_pack` bin; diagnostic render varies offsets and reports metrics | Reads source samples, but does not persist a product decision or prove the musician path | Diagnostic scaffold; risk of being mistaken for shipped product intelligence | **Do not promote unchanged.** RIOTBOX-1432 may reuse small established feature calculations, then delete or clearly retire overlapping diagnostic selection code. |
| MC-202 source phrase selection | Source-phrase features, seven typed candidate families, scoring, stay-out, Session plan, and renderer; `P` chooses bass, answer, instigator, or silence | Source features change candidate family, cells, role, and output; extensive deterministic and cross-source tests. Beat08 bass was human-rejected, after which source evidence correctly removed bass ownership | Product algorithm with high complexity and no general human bass pass; risk is further weight tuning without musician value | **Freeze, do not expand.** Keep source-derived and `stay_out` behavior. RIOTBOX-1043 owns later cross-family human quality; no new MC-202 ticket follows this audit. |
| TR-909 support, fill, slam, and takeover | Typed core render policy plus callback renderer; `s`/`f` produce immediate drum/transient lift and destructive contrast | Source section/transient evidence can change support profile and levels; queue/commit, replay, exact live path, and structured human pass exist | Product algorithm; fixed primitive recipes are explicit and gesture-owned | **Retain.** It is the hardest accepted active layer and its complexity has audible proof. |
| Live character, lane ownership, mixer balance, and scene movement | `derive_live_performance_policy`, Scene projection, Session/replay, RuntimeMix; `F`, `P`, `y`, and `Y` select lead, restraint, contrast, and return | Source phrase/section evidence changes dense/tonal/sparse character, bass ownership, lane levels, and scene direction; dense exact-live pass plus tonal/sparse held-loop reviews | Product policy; coarse sidecar sections limit the strength of any broad intelligence claim | **Retain accepted behavior.** Do not add thresholds until a named live source exposes a failure. |
| Percussive-force F1–F4 | Isolated Stage-A research modules and matrix binaries | Mechanically bounded, multi-source, and human-reviewed negative evidence; no RuntimeMix ownership | Frozen diagnostic research; repeated pursuit would displace higher-value product work | **Retire from the active priority lane.** Preserve history; do not import or retune it. |
| Realtime scheduling, queue/commit, and replay | Action Queue, commit boundaries, runtime snapshots, Session and replay | Typed actions and commit records drive exact output; broad regressions and exact live review exist | Product spine; timing bugs directly break audible correctness | **Retain.** No competing state system was found. |
| Unavailable/degraded decisions | Timing readiness, missing source/capture refusal, MC-202 `stay_out`, W-30 digital silence | Tests plus reviewed weak/bad-timing product states prove no synthetic musical fallback | Product safety behavior; misleading diagnostic controls remain a labeling risk | **Retain fail-closed behavior.** `FallbackControl` remains rejected/control-only and produces no product phrase. |

## High-Complexity Retention Tests

The retained complex algorithms have concrete falsifiers:

- **Source Timing:** mismatched or unconfirmed timing must refuse a trusted
  source window; explicit BPM/downbeat identity must survive Session/replay.
- **MC-202 phrase selection:** neutralizing the relevant source features must
  change the family or produce `stay_out`/silence; filename changes must not
  change it. No further scoring revision is justified without a fresh human
  musical pass on a trusted source.
- **Live performance policy:** changing source evidence across dense, tonal,
  and sparse material must change lead/restraint/ownership while preserving
  exact-path headroom and replay. A single fixed lane mix across sources fails.
- **TR-909 policy and realtime scheduling:** committed `s`/`f` gestures must
  change the declared local drum/transient or arrangement window at the landed
  boundary; log-only or late changes fail.

## RIOTBOX-1432 Frozen Handoff

RIOTBOX-1432 should use this bounded comparison:

- **Baseline:** the current one-bar `CaptureSourceWindow` selected solely by
  the musician/transport boundary; the Recipe 17 diagnostic starts at source
  bar 1. The existing opening-loop candidate and bin-local eight-slice scorer
  are not a stronger product baseline.
- **Development matrix:** exactly the registered positive sources
  `Beat03_130BPM(Full).wav` (`dense_break`), `DH_RushArp_120_A.wav`
  (`tonal_riff`), and `DH_BeatC_KickSnr_120-01.wav` (`sparse_drums`). Use their
  existing explicit timing-confirmation contracts. Do not use Beat20 as
  positive evidence; it remains weak/bad-timing negative coverage.
- **Candidate limit:** at most two explainable source-evidence policies plus
  the baseline. One may rank timing-valid bar/subwindow candidates by
  onset/attack-body and local spectral contrast; one may add bounded
  repetition/salience and phrase contrast. Exact equations and constants must
  freeze in RIOTBOX-1432 before comparative rendering.
- **Product ownership:** keep the existing
  `CaptureBarGroup -> W30AuditionRawCapture -> PromoteCaptureToPad ->
  W30TriggerPad` path. Persist a typed selection policy/version, selected source
  range, evidence values, and reason with the capture lineage so replay makes
  the same choice without reanalysis. No new action system is needed.
- **Falsification:** same source/evidence must be deterministic; neutralized
  evidence must change or reject selection; path/filename changes must not;
  all three sources must remain non-silent, unclipped, recognizably distinct,
  and inside the accepted timing/runtime path.
- **Human gate:** only after all three Development sources pass technical
  comparison, present one bounded blinded Beat03 baseline/candidate comparison
  through the exact product RuntimeMix path. Judge hook memorability, source
  identity, physical attack where relevant, arrangement usefulness, and stage
  triggerability. `Different but not better` rejects the new policy.

The implementation diff must contain more product/audio ownership than
ticket-specific validation code. Reuse existing capture, render, replay, and
listening harnesses.

## Prioritized Actions

1. **RIOTBOX-1432 — do now.** Replace the feature-independent W-30 hook choice
   with the bounded product-owned selection above and require a fresh human
   pass. Expected benefit: the instrument finds a stronger keeper instead of
   merely processing whichever bar was captured first.
2. **No second implementation ticket.** Keep RIOTBOX-1033/1042/1043, 1399,
   1412, and 1420 in their existing backlog roles. None currently blocks the
   W-30 hook comparison.
3. **Stop the isolated force lane.** F1–F4 and the natural-control result stay
   frozen negative evidence. A future force ticket requires a genuinely new
   research basis; it is not the next P023 priority.

The audit recommendation was derived without opening or rendering source audio,
and no human playback was needed. The normal repository CI subsequently
exercised its registered non-holdout regression fixtures; those renders did not
inform the recommendation. No validator or benchmark framework was added.
