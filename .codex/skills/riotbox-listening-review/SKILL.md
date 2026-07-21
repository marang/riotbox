---
name: riotbox-listening-review
description: Riotbox structured human listening-review workflow for audible PRs, RIOTBOX-1114 review packs, human_verdict handling, automated musical fitness interpretation, and musician-facing taste verdicts.
---

# Riotbox Listening Review

## Operating Rule

Use this skill when Riotbox work changes audible behavior and needs a human taste verdict, review pack, or PR note about why `human_verdict` remains `unverified`.

Structured listening review complements automated musical fitness. Treat automated musical fitness as a regression and collapse detector, not a human approval substitute.

## Workflow

Use Riotbox's repo-local workflow instead of ad hoc notes:

```bash
just listening-review-pack RIOTBOX-123
just listening-review-record ...
just listening-review-fixtures
```

The canonical implementation is `scripts/listening_review_workflow.py`; the contract is documented in `docs/specs/audio_qa_workflow_spec.md`.

## Pre-Playback Brief

Before every human playback, technically preflight the exact WAV or A/B
artifact that will be played; do not infer its behavior from a sibling render
or report. Verify and interpret:

- artifact paths and A/B segment order, plus hashes or sample-exact segment
  identity when identity or assignment could be ambiguous
- `ffprobe` format, sample rate, channels, duration, and frame count when
  available
- peak, RMS, LUFS when available, silence, and clipping
- role-appropriate time-local absolute and relative deltas plus waveform
  correlation; whole-render aggregates alone are insufficient
- frequency-domain deltas chosen for the expected role, such as low-end,
  drum body, attack, or hook bite

State what objectively changed, what stayed unchanged, whether the intended
role survives the full mix, and whether the artifact is correctly assigned. If
the artifact is invalid, misassigned, or demonstrably too similar for its
claimed effect, fix, regenerate, or label it weak before requesting a human
verdict. Metrics screen and explain the sample; they do not replace listening.

Only after that technical gate, provide a compact factual brief that states:

- the playback context: isolated stem, full mix, source, baseline, or comparison
- the selected product role or candidate family
- the intended musical function and expected audible effect
- important properties that are explicitly not expected, such as bass pressure
  from an answer / punctuation stem
- the dimensions the listener should judge for this artifact

Before calling any playback isolated, enumerate all callback/mixer
contributors that can remain audible in that state. Source monitor, internal
resample taps, support lanes, diagnostic voices, and stopped manual previews
count as contributors even when their route is called `internal`. Silence every
unclaimed contributor or label the artifact as a composite; do not ask the
listener to judge an isolated hook from a hook-plus-scaffold mix.

When the performance preset promotes a captured source hook, reject an
unintentional raw-source `Blend` underneath that same material if the two paths
restart from different source phases. Use the preset's declared monitor route
for the candidate, and reserve Source/Blend for explicitly labeled A/B checks.

Never use `pressure` as an unqualified listening target. Name the intended
domain and its owner:

- `bass / low-end pressure`: audible sub or bass weight, movement, and
  kick/bass support from the typed bass owner
- `drum / transient pressure`: kick or snare punch, attack, body, and physical
  impact
- `midrange / hook aggression`: bite, distortion, density, or stab/chop force
- `arrangement / performance impact`: contrast, drop, silence, escalation, or
  return payoff

State which domains are expected and which are not. If an artifact targets
bass pressure but produces no recognizable bass, record that as a bass-pressure
failure; do not substitute drum punch, loudness, or general bus energy and call
the target successful.

Keep the brief factual and do not prime the listener toward a positive verdict.
If role assignment is unresolved or contradictory, do not request a taste
verdict; fix or surface the decision first.

For interactive local playback, show a conspicuous listening-check line and
wait for an explicit readiness confirmation before each playback. A previous
confirmation does not carry over to the next artifact or repeat. Treat any
playback started without that confirmation as unheard and do not record a human
verdict from it.

Bound every playback before requesting readiness:

- default maximum: 10 seconds
- isolated capture or stem: 2-5 seconds when sufficient
- longer playback: only for a named multi-bar development claim, with exact
  duration and purpose stated in advance

Do not let a live instrument loop stand in for a bounded review artifact.
Schedule an explicit audible stop at the announced endpoint, verify that all
active lanes are silent, and terminate the runtime immediately if transport
stop does not silence them. Confirm silence before preparing the next sample.

For a live multi-stage review, drive queued actions ahead of their intended
quantized boundaries and validate the landed observer commit beats before
requesting readiness. Wall-clock sleeps alone are not timing evidence. Reject
the take if any stage misses its declared beat/bar interval or if the explicit
transport stop misses the announced endpoint; do not transfer a verdict from a
different arrangement duration even when the sound recipe is identical.

Do not turn non-discriminative technical reruns into human listening work. If
the sound recipe is intentionally unchanged and only observer timing, capture,
or transport proof differs, validate that difference mechanically. Request a
new human verdict only when the preflight demonstrates a material audible or
musical question. Repeated reports of "same" are a stop signal: end the
comparison, preserve any existing artifact-bound verdict, and resolve the QA
question without listener fatigue.

## Primitive Vocabulary Provenance

Do not collapse fixed instrument vocabulary and fallback output into one label.
A typed, versioned primitive may be valid product output when an explicit
committed performer gesture owns it. It is still not source-derived musical
intelligence unless source evidence actually selects or composes the recipe.

For an audible product primitive, require the listening manifest to record:

- `pattern_origin: primitive_renderer`
- the versioned primitive schema, exact recipe ID, and non-empty typed selection
  inputs
- `evidence_role: product_primitive_vocabulary` with
  `product_output_allowed: true`
- a versioned boundary that distinguishes
  `recipe_derivation_claimed: false` and
  `pattern_selection_claimed: false` from any truthful
  `source_output_modulation_claimed: true`; source modulation must name the
  exact source feature, derived policy values, resolved render inputs, and
  affected RuntimeMix parameters
- `source_failure_fallback: false`
- a JSON-pointer activation reference that resolves to the committed command,
  action ID, and boundary, plus affected RuntimeMix paths and artifacts that are
  actually declared by the manifest
- a primitive schema explicitly registered for product output in the shared
  validator, with schema-specific recipe/input, source-modulation, activation,
  RuntimeMix, focus-path, and candidate-WAV checks; regex-shaped identifiers
  alone are insufficient
- `quality_proof: false`, `demo_readiness: unverified`, and promotion blocked
  specifically for `source_derived_musical_intelligence`

Keep diagnostic primitives `non_product_diagnostic_control` with product output
forbidden. Never use the product-vocabulary role for an automatic substitute
when trusted source behavior is unavailable.

## PR Rule

For audible PRs, state briefly whether a structured listening pack/verdict exists or why `human_verdict` remains `unverified`.

If a review says the output is technically valid but musically weak, convert that into one concrete follow-up: source selection, chop policy, drum pressure, bass movement, contrast/drop behavior, fixture threshold, or UI cue.

## Verdict Discipline

Do not claim a musician-facing pass from metrics alone. A good review should identify:

- the strongest audible element
- source recognition or source masking
- whether a hook appears within two bars
- the main musical failure, if any
- a concrete preferred direction
- what to avoid repeating
