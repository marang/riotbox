# RIOTBOX-1578 — loaded recorder observation adapter

Date: 2026-10-15. Base: `6edcef2cdd8fbde889ca2e822635442e836f87ee`.
Scope: source-free implementation and generated verification only.
Contract: [loaded observer V1](../benchmarks/loaded_recorder_observation_v1.md),
RBX-444. No actual audio runtime, device, service or host-audio metadata operation.

## Delivered boundary

The existing recorder's optional typed observation samples completion in memory,
preserves immediate stop and verifies Stopped before emitting the milestone.
Finalization, WAV/proof publication and Session save still follow. No observer
means no clock reads or marker construction. Missing/reversed clocks remain
unqualified; observer failures stay latched without invalidating a valid product
capture. The marker is explicitly not a success event.

The closed loaded workload reuses the existing isolated operator, private child
environment, sink/node lifetimes, watchdog, signals, route/stream/group cleanup
and bound ALSA measurement leaf. It admits only exact generated V1 preparation
identities, allocates byte-identical sibling run Sessions and declares outputs.
Immediately before hydration it rechecks the registered case, exact initial
Session bytes and absent outputs, then consumes that case once. Baseline files
remain immutable; this is not a hostile concurrent-writer guarantee.

Explicit CLOCK_MONOTONIC query intervals distinguish premature, ambiguous and
eligible route disappearance. Only safe absence can await a delayed milestone;
foreign/replaced routes and default-state changes reject immediately. Existing
deadlines bound actual process/stream drain. Independent artifact analysis runs
only after verified cleanup and does not extend live output or drain deadlines.
Computed artifact evidence is exclusively published before a later rejection.
Final success also requires exact final observer/stdout agreement, actual
WAV/proof/Session/action/receipt identities and technical gates, ALSA attribution,
and the existing Core metadata-readiness validator. No product model is copied.

## Findings repaired before PR

1. **P1, Spec And Evidence Auditor / access boundary:** an initially valid
   mutable run copy could be changed before CLI hydration while immutable-input
   checks still passed. The exact registered one-use prelaunch guard now rejects
   changed/redirected/missing/linked copies and occupied outputs before launch.
2. **P2, Spec And Evidence Auditor:** computed failed artifact metrics could
   disappear into an exception message. Owned `.artifacts.json` records now
   retain computed values before failure or cleanup wrapping; result evidence
   declares case paths and the active run prefix. Collision targets are preserved.
3. **P3, Spec And Evidence Auditor:** clipped finite samples were omitted from
   retained RMS energy. All finite samples now contribute; invalid overflow or
   nonfinite payloads explicitly mark RMS unavailable instead of inventing zero.
4. **Coordinator integration, API/consumer compatibility:** actual Core Session
   spells the fixed native decode profile `native`, while Graph uses `Native`.
   Exact supported spellings are checked, without generic case normalization.
   Report-promoted f32 capture fields also differ textually from saved Session
   f32 spellings. Only declared typed f32 leaves compare by exact f32 bits;
   all IDs, hashes, paths and remaining fields retain strict checks. New numeric
   regressions were executed RED at the previous equality and then GREEN.
5. **Coordinator self-review, correctness:** the actual generated preparation's
   confirmed bar phase requests beat 23, not a multiple of four from beat zero.
   Marker admission no longer invents an absolute-zero anchor. Final proof and
   Core retain their existing confirmed-anchor alignment checks.

Independent refreshes closed findings 1–3 and checked the compatibility repairs.
No retained finding remains. Maintainer/Rust/adversarial, Spec/Evidence and
Product Pragmatist/Risk lenses were applied; authors never received independent
credit for their own code. The cohesive existing generated controller fixture
remains 609 lines; new observation regressions have a real semantic child module.
No mechanical include split, public Audio test constructor or App state was added.

## Verification

- 18 actual shared-controller lifecycle tests pass, including six new observation
  tests. The 80-test live-master family overlaps these and is not additive.
- Generated Python controls cover fixture identities/path confinement, exact
  copies, typed f32 and prelaunch admission; observer schemas/clock/lineage;
  route timing, watchdog, defaults, cleanup and diagnostic failures; all six
  2/8/16-bar × 44.1/48 kHz published geometries; tampered proof/receipt/Session/
  WAV; retained failed metrics; closed workload and Core-readiness integration.
- All 165 existing silent-host controls pass, preserving default V3/V4 behavior
  and consumed-phase/disabled-CLI fences.
- Full source-free `just ci` passes, including Rust tests, generated audio smokes,
  Python/contract controls and strict all-target/all-feature Clippy. Final loaded
  controls recheck late admission/postflight repairs after that complete run.
- A fresh actual generated V1 producer completed six offline windows. Its Python
  admission and three byte-identical sibling copies pass; Core's metadata-only
  CLI restores all copies with zero recording receipts, as expected. Original
  producer hash was checked before CI rebuilt the mutable target path; the
  original observed binding was retained. A changed target hash correctly rejects.
- Native PR CI and remote review remain separate merge gates, recorded with the
  exact reviewed head in PR/Linear closeout, not inferred from local tests.

Local logs: `/tmp/riotbox-1578-full-ci.log`,
`/tmp/riotbox-1578-loaded-final-snapshot.log`,
`/tmp/riotbox-1578-silent-regressions.log`,
`/tmp/riotbox-1578-rust-milestone-final.log`,
`/tmp/riotbox-1578-rust-live-master-regressions.log`,
`/tmp/riotbox-1578-fresh-generated-producer.log`,
`/tmp/riotbox-1578-fresh-admission.log`,
`/tmp/riotbox-1578-fresh-copies.log`,
`/tmp/riotbox-1578-fresh-core-copy-readback.log`, and independent
`/tmp/riotbox-1578-independent-*.log` / `/tmp/riotbox-1578-review-*.log`.
Counts from repeated/overlapping refreshes must not be summed.

## Claim and next boundary

This completes the source-free adapter, not host qualification or a musical
feature. No new DSP, runtime, action, persistence/replay authority, production
threshold, rate/buffer/priority policy, source selection or recording version.
No real source/capture/Holdout/commercial audio, source-directory discovery,
old ignored attempts, playback, DAW, broad QA or Windows expansion.
The original ALSA trigger remains unexplained; RIOTBOX-1041/1036 remain open.

Next is a separately prospectively bound actual loaded phase, not another
adapter prerequisite: exact clean reviewed revision/native CI/binaries/protocol,
fresh generated preparation/attempt owners and fresh explicit authorization.
Existing host phases remain consumed and cannot authorize or substitute for it.
