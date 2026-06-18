# MC-202 Producer-Grade Closeout v1

Status: active P023 closeout gate  
Ticket: RIOTBOX-1279  
Parent quality track: RIOTBOX-1264

## Purpose

This benchmark closes the current MC-202 implementation slice without
pretending that unreviewed audio is already producer-grade.

It aggregates:

- the professional output listening pack
- the dense/non-dense MC-202 real-source listening pack
- the MC-202 source-composed review gate
- review candidate hashes, prompts, source families, and human verdict state

The gate passes only when the MC-202 material is technically reviewable across
dense and non-dense sources while still blocking demo-bank promotion and product
quality claims until structured listening records a human verdict.

## Command

```bash
just mc202-producer-grade-closeout-smoke
```

The default output is:

```text
artifacts/audio_qa/local-mc202-producer-grade-closeout/
```

Primary artifacts:

```text
artifacts/audio_qa/local-mc202-producer-grade-closeout/mc202-producer-grade-closeout.json
artifacts/audio_qa/local-mc202-producer-grade-closeout/mc202-producer-grade-closeout.md
```

## Contract

The report schema is `riotbox.mc202_producer_grade_closeout.v1`.

The validator requires:

- `technical_closeout_result: pass`
- `producer_grade_promotion_result: blocked_for_human_promotion`
- `quality_claim_allowed: false`
- `demo_bank_promotion_allowed: false`
- `parent_ticket_state: keep_open`
- at least one dense and one non-dense MC-202 review candidate
- source-composed evidence for the dense and sparse MC-202 candidates
- any primitive/template-only candidates preserved as promotion blockers
- real-source primitive controls marked as non-product output
- all review candidates still `human_verdict: unverified`,
  `demo_readiness: unverified`, and `quality_proof: false`

Mutation fixtures reject premature quality claims, premature promotion,
stale human verdict state, primitive-control leakage into product output, and
missing primitive/template promotion blockers when a candidate regresses to
primitive/template-only.

## Boundary

This is a closeout gate, not a musical pass oracle.

It is useful to the software because it gives CI one stable place to say:
"MC-202 is ready for structured listening, but not ready for demo-bank
promotion." It is useful to the musician because weak, template-like, or
unheard bass behavior cannot be hidden behind a green render.
