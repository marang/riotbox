# Release-Grade Musician Demo Bank Spec

Version: 0.1
Status: Draft
Audience: audio, QA, product, agents

---

## 1. Purpose

The release-grade musician demo bank is the curated index of Riotbox outputs
that can be referenced in PRs, roadmap notes, and release work. It does not
replace listening review, and it must not promote generated artifacts just
because they render.

The machine-checkable contract lives in
`scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json` and is validated
by:

```bash
just release-grade-demo-bank-fixtures
just demo-bank-promotion-fixtures
```

---

## 2. Entry Contract

Each demo-bank entry records:

- source family and source WAV path
- rendered WAV path and SHA-256 identity
- metrics/report path and SHA-256 identity
- review prompt path and SHA-256 identity
- `human_verdict`: `pass`, `weak`, `fail`, or `unverified`
- `demo_readiness`: `demo_ready`, `not_demo_ready`, or `unverified`
- a short musician-facing demo-worthiness note
- the seven sound-product rubric dimensions
- fix categories for weak or failed examples

The bank must include at least one dense-break entry and at least one
non-dense-break entry before it is considered structurally complete.

---

## 3. Claim Rules

Only `human_verdict: pass` entries may be `demo_ready`.

Weak and failed entries remain in the bank only when they teach a concrete fix
category such as source selection, chop policy, drum pressure, bass movement,
mix bus, destructive gesture, fixture threshold, or UI cue.

`human_verdict: unverified` entries can be indexed as candidates, but they must
stay `demo_readiness: unverified` and must not claim product quality.

---

## 4. PR Usage

PRs may reference demo-bank evidence when they name:

- the demo-bank entry id
- the source family
- the rendered WAV and metrics/report identity
- the human verdict and demo readiness
- the short reason the example is demo-worthy or not demo-worthy yet

If an audible PR references only generated, scripted, or unverified candidate
artifacts, the PR must say `human_verdict: unverified` and avoid demo-ready or
release-ready language.

---

## 5. Listening Review Promotion

Structured `riotbox.listening_review.v1` reviews can be promoted into the demo
bank with `scripts/promote_listening_review_to_demo_bank.py` only when the review
carries `audio_judge_label` metadata from the professional listening pack.

Promotion requires:

- artifact identity for the performance report, agent review, source window,
  full-performance WAV, and review prompt
- source family and source path
- rendered WAV path and SHA-256
- metrics/report path and SHA-256
- review prompt path and SHA-256
- human verdict mapped from the structured review
- a musician-facing demo-worthiness note
- the seven demo-bank musical summary fields

`keep` maps to `pass` and becomes `demo_ready`. Weak and failed reviews may be
preserved only as `not_demo_ready` entries with concrete fix categories.
`unverified` and `inconclusive` reviews must not be promoted into demo-ready
entries. When `--require-artifact-hashes` is used, stale or missing artifact
hashes block promotion.

---

## 6. Non-Goals

This spec does not create a taste oracle, a runtime demo generator, or a public
marketing page. It is a repo-owned evidence index for curated musician examples
and weak-output learning.
