---
name: riotbox-listening-review
description: Riotbox structured human listening-review workflow for audible PRs, RIOTBOX-1114 review packs, human_verdict handling, automated musical fitness interpretation, and musician-facing taste verdicts.
---

# Riotbox Listening Review

## Operating Rule

Use this skill for human taste verdicts, review packs, or PR notes explaining why `human_verdict` remains `unverified`. Fitness and fixtures detect collapse; they never award a musician-facing pass.

Read the [audio-QA router](../../../docs/specs/audio_qa_workflow_spec.md) before
every playback task. It alone owns playback/verdict detail and the
product-vocabulary/source contract.

## Workflow

Use the repo workflow, implemented by the
[listening-review workflow](../../../scripts/listening_review_workflow.py):

```bash
just listening-review-pack RIOTBOX-123
just listening-review-record ...
just listening-review-fixtures
```

## Safety Gate

- Preflight the exact artifact; invalid, unresolved, or misleading evidence must
  not be played for a verdict.
- Give a factual, non-priming brief and inventory every audible contributor.
- Require fresh readiness for every playback; unconfirmed playback is unheard.
- Enforce the contract's bound, audible stop, verified silence, and leak kill.
- Use committed observer timing, not sleeps, and never transfer a verdict.
- Technical-only reruns do not need listening; repeated “same” is a stop signal,
  and the generation limit requires listening or explicit handoff.

## Result

Record the artifact-bound verdict, strongest element, source/hook result, main
failure, direction, avoid-list, and one concrete follow-up. Every audible PR
states whether this verdict exists or why it remains `human_verdict: unverified`.

Translate informal listener comments into concise, professional evidence
language in durable documentation and structured summaries. Preserve the
comment's meaning, certainty, and severity; do not embellish or weaken it.
Quote the listener verbatim only when the exact wording is itself material.
Do not edit an already hash-bound review artifact solely for editorial
normalization; normalize its meaning in the durable record that cites it.
