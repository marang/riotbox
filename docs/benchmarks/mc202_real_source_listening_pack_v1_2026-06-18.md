# MC-202 Real-Source Listening Pack v1

Status: active P023 listening-review scaffold  
Ticket: RIOTBOX-1278

## Purpose

This benchmark generates dense and non-dense real-source MC-202 listening packs
without promoting scripted diagnostics to product quality proof.

Each case renders a local real source through the Feral-grid pack and records:

- source window
- MC-202 stem
- generated-support mix
- listening-review pack
- MC-202 expression summary from source timing and source contour evidence
- selected MC-202 motif metadata
- per-case `mc202_role_evidence` that tells the reviewer whether the MC-202
  candidate should be judged as bass pressure, pressure-answer, or
  hook-restraint / stab-answer material
- primitive A/B control metadata marked as non-product evidence

The primitive control is not fallback music. It exists only to show that the
source-contoured MC-202 result differs from the disabled primitive/silent control
path. Product output must still use source-backed MC-202 evidence or visible
unavailable / degraded state.

## Command

```bash
just mc202-real-source-listening-pack-smoke
```

The default output is:

```text
artifacts/audio_qa/local-mc202-real-source-listening-pack/
```

The primary report is:

```text
artifacts/audio_qa/local-mc202-real-source-listening-pack/mc202-real-source-listening-pack.json
```

## Contract

The report schema is `riotbox.mc202_real_source_listening_pack.v1`.

The validator requires:

- at least one dense and one non-dense real-source case
- `human_verdict: unverified`
- `quality_proof: false`
- source timing and source-contour expression fields
- selected MC-202 motif fields
- `mc202_role_evidence` with source-family-matched role targets:
  sparse bass-pressure sources require `bass_pressure`, dense/non-dense sources
  require `pressure_answer`, and tonal-hook sources require
  `hook_restraint_stab_answer`
- role evidence that remains `quality_proof: false` and
  `human_verdict: unverified`, with `proof_scope: listening_review_target`
- non-silent MC-202 stem evidence
- primitive A/B control with `product_fallback_allowed: false`
- source-contour A/B delta above the required threshold

Mutation fixtures reject accidental quality claims, missing expression evidence,
product-fallback controls, silent MC-202 stems, missing/stale role evidence,
and source-family-inappropriate MC-202 role targets.

## Boundary

This benchmark is a listening-review scaffold. It can prove that MC-202 evidence
is present and reviewable across dense and non-dense real sources, but it cannot
claim demo readiness or product quality until structured human listening records
an accepted verdict.
