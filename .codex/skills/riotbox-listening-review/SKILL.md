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

Do not play a review artifact without first telling the listener what the
artifact is intended to do. Before every playback, provide a compact brief
that states:

- the playback context: isolated stem, full mix, source, baseline, or comparison
- the selected product role or candidate family
- the intended musical function and expected audible effect
- important properties that are explicitly not expected, such as bass pressure
  from an answer / punctuation stem
- the dimensions the listener should judge for this artifact

Keep the brief factual and do not prime the listener toward a positive verdict.
If role assignment is unresolved or contradictory, do not request a taste
verdict; fix or surface the decision first.

For interactive local playback, show a conspicuous listening-check line and
wait for an explicit readiness confirmation before each playback. A previous
confirmation does not carry over to the next artifact or repeat. Treat any
playback started without that confirmation as unheard and do not record a human
verdict from it.

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
