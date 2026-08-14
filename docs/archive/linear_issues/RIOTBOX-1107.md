# `RIOTBOX-1107` Implement automated musical fitness validator and schema

- Ticket: `RIOTBOX-1107`
- Title: `Implement automated musical fitness validator and schema`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1107/implement-automated-musical-fitness-validator-and-schema`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1107-automated-musical-fitness-validator`
- Linear branch: `feature/riotbox-1107-implement-automated-musical-fitness-validator-and-schema`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `benchmark`, `review-followup`
- PR: `#1087 (https://github.com/marang/riotbox/pull/1087)`
- Merge commit: `0bd53b60f0623bcc5ef4c386180858acd14a1ecb`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1087; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Why

Riotbox needs a code-driven anti-bad-output gate that catches technically valid but musically weak generated audio before every slice depends on human listening.

## Scope

Add a product-facing automated musical fitness validator for generated audio artifacts. It should analyze existing render/showcase outputs and emit a stable JSON contract, not alter realtime rendering.

## Required Contract

Emit schema `riotbox.automated_musical_fitness.v1` with:

* `technical_status`
* `automated_musical_fitness_status`
* `result`
* `selected_candidate`
* `failure_codes`
* `score_breakdown`
* `human_verdict: unverified`

## Checks

Cover at minimum:

* silence / near-silence / clipping sanity
* fallback collapse or byte/metric identity collapse
* source relation vs source-masked or source-fake output
* variation and non-static movement over the loop
* lane balance so one weak or placeholder lane does not dominate
* low-end / transient presence suitable for Riotbox's beat-forward direction

## Acceptance

* Validator passes on a known-good synthetic/representative fixture.
* Validator fails deterministic bad fixtures with explicit failure codes.
* Output is token-bounded and suitable for CI logs.
* The validator does not claim human-approved musical quality.

## What Shipped

- Closed the bounded scope: Implement automated musical fitness validator and schema.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
