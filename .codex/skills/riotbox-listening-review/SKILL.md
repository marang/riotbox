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
