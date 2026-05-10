# P011 Exit Readiness Decision 2026-05-10

Context:

- ticket: `RIOTBOX-712`
- preceding evidence: `docs/reviews/p011_exit_evidence_gate_review_2026-05-10.md`
- phase: `P011 | Pro Hardening`
- review mode: phase-readiness decision

## Decision

P011 can close as the bounded MVP-spine hardening phase.

The passing aggregate evidence gate is sufficient for the P011 boundary as it is
currently documented: replay, recovery, export reproducibility, and stage-style
stability are covered by CI-safe, deterministic, machine-checkable evidence.

Primary development attention can now move to `P012 | Source Timing Intelligence`.

## Why This Is Enough

P011 was explicitly scoped as final MVP-spine hardening, not the full 1.0 release
or live-performance certification phase.

The current gate proves the hardening spine at the right level:

- replay evidence executes through the replay-family manifest and stage-style
  snapshot convergence proof
- recovery evidence executes through generated startup observer drills plus the
  recovery surface test family
- export evidence executes through deterministic product-export reproducibility
  smoke with stable generated-support audio hashes
- stage-style reliability evidence executes through repeated generated
  observer/audio stability proof with stable full-mix hashes

The gate is not aspirational: `just p011-exit-evidence-gate` ran locally and
passed all 7 referenced commands on 2026-05-10.

## What Remains Out Of P011

The following are still real product needs, but they are not blockers for closing
P011 as bounded MVP hardening:

- host-audio soak and multi-hour endurance belong to `P017 | Live Performance Readiness`
- full arrangement, stem package, and live recording export belong to
  `P016 | Pro Workflow / Export`
- deeper generated musical quality belongs to `P013 | All-Lane Musical Depth`
- timing-aware source-grid work belongs to `P012 | Source Timing Intelligence`
- Ghost autonomy beyond Watch / Assist belongs to `P018 | Ghost + Feral Autonomy Expansion`

Keeping these out of P011 is important. Otherwise P011 becomes a permanent
catch-all hardening bucket and blocks the timing work that Riotbox now needs most.

## Next Development Track

Start P012 from `docs/plans/source_timing_intelligence_plan.md`.

The first P012 work should stay Rust-first and contract-oriented:

- preserve Source Graph, session, action, observer, and audio-QA seams
- improve source timing confidence and anchor evidence without adding a parallel
  analysis architecture
- prove output alignment with source-vs-output evidence for TR/Kick-Bass,
  MC-202, and W-30 one bounded slice at a time

## Operational Notes

- Keep the P011 aggregate gate available and green as a regression gate.
- Do not delete or weaken P011 evidence just because the phase is closing.
- If future P012/P013 work breaks replay, recovery, export reproducibility, or
  stage-style stability, treat that as a regression against the closed P011
  spine.
