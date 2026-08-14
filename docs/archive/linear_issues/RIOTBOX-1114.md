# `RIOTBOX-1114` Add interactive listening-review workflow with structured human feedback

- Ticket: `RIOTBOX-1114`
- Title: `Add interactive listening-review workflow with structured human feedback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1114/add-interactive-listening-review-workflow-with-structured-human`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1114-listening-review-workflow`
- Linear branch: `feature/riotbox-1114-add-interactive-listening-review-workflow-with-structured`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `benchmark`, `review-followup`, `workflow`
- PR: `#1092 (https://github.com/marang/riotbox/pull/1092)`
- Merge commit: `c021dd10810e4dad8891092992315150d8751402`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1092; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Why

Automated audio QA can reject broken, static, silent, fallback-collapsed, or source-fake output, but it cannot certify that generated audio is musically useful. Riotbox needs a repeatable human listening loop that produces audio examples, asks focused questions, records structured feedback, and feeds that feedback back into future example generation and code review.

## Scope

Add an interactive listening-review workflow for audio-producing Riotbox slices. The workflow should generate a small review pack, guide the human through one-question-at-a-time listening prompts, store the verdict as structured data, and make that verdict available to review and future generation.

## Proposed Commands / Artifacts

Introduce commands equivalent to:

* `just listening-review-pack TICKET=RIOTBOX-123`
* `just listening-review-record TICKET=RIOTBOX-123 REVIEW=...`

Expected review pack contents:

* source audio when relevant
* before / after or candidate WAVs
* compact metrics JSON
* expected audible behavior in musician language
* markdown prompt for the listening session
* structured verdict JSON under a local audio QA artifact path

## Required Verdict Fields

At minimum capture:

* ticket / PR / command / source file / seed or config
* `technical_status`
* `automated_musical_fitness_status`
* `human_verdict`: keep / reject / technically_ok_but_musically_weak / inconclusive
* strongest element: kick / snare / bass / stab / chop / vocal / silence / none
* source-recognition verdict
* hook verdict after two bars
* failure reason
* preferred direction
* avoid list
* concrete follow-up

## Review Integration

Extend Riotbox audio review guidance so PRs that affect audible behavior must say whether a listening review pack exists, whether the human verdict is recorded, or why the change remains `human_verdict: unverified`.

## Boundaries

This must not become CI-only, memory-only, or a second source of truth. Human verdicts should be local/repo artifacts suitable for later archival, not hidden agent memory. The workflow should complement automated musical fitness gates and must not claim objective taste certainty.

## Acceptance

* A human can run one command, hear generated examples, and answer focused questions.
* The answers are stored as structured feedback that future agents can inspect.
* Code review can require or reference the listening verdict for audible changes.
* The workflow distinguishes automated pass from human approval.
* At least one dry-run fixture proves verdict parsing and report shape without needing audio hardware.

## What Shipped

- Closed the bounded scope: Add interactive listening-review workflow with structured human feedback.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
