# Riotbox Workflow Conventions

Version: 0.4
Status: Active
Audience: contributors, reviewers, coding agents

---

## 1. Purpose

This is the canonical operational core and router. It owns cross-system order,
mandatory gates, continuation, and stop conditions. Focused procedure lives in:

- [GitHub, PR, Review, And CI](./workflow/github_pr_ci.md)
- [Linear Lifecycle](./workflow/linear_lifecycle.md)
- [Archive And Cleanup](./workflow/archive_cleanup.md)
- [Context And Token Hygiene](./workflow/context_hygiene.md)

Read this core before implementation work, then load only the module needed for
the current operation. The roadmap owns product direction, `AGENTS.md` owns
non-negotiable guardrails, and Riotbox skills/specs own product, audio, QA, and
musician judgment. Modules own operational detail; if one conflicts with this
core, follow the core and update both.

---

## 2. Core Rule

Workflow priority order, from highest to lowest:

1. Preserve user work and avoid destructive operations.
2. Keep product truth in repo docs/specs, Git history, and Linear; do not
   leave execution state only in chat or local memory.
3. Keep the Linear issue loop honest before touching Git branches: exactly one
   issue must represent the active slice and must be `In Progress` before a
   feature branch is created or reused for implementation.
4. Keep PR/CI/review/merge gates intact for every normal implementation slice.
5. Continue autonomously when requested, but only through the same Linear-first
   workflow loop. "Do not stop" is never permission to skip Linear, branch
   naming, PR, CI, review, or closeout state.

Default workflow:

`Linear issue -> branch -> scoped commit(s) -> PR -> review -> merge -> sync local main -> close out ticket and branch`

Do not skip the PR step for normal feature or implementation work.
Do not skip or backfill the Linear issue step for normal feature or
implementation work.

This operational loop is the execution form of the roadmap's core delivery loop:
each bounded ticket should move one spec/research/vertical-slice/test/benchmark
step forward, then leave a clean review and closeout trail before the next slice.

When the user explicitly requested phase-level or cross-ticket autonomous
continuation, ticket closeout is a workflow transition, not a stop condition.
For a bounded one-ticket request, verified closeout completes the requested
scope unless the user asks for the next ticket too.

While explicit cross-ticket continuation remains active, the agent must enter
the next workflow state after closeout:

1. verify local `main` is clean and synced
2. inspect whether any open PR or CI run needs action
3. if an open PR is red and the failure belongs to that slice, fix it
4. if an open PR is green and mergeable, merge it and close it out
5. if no active PR is blocked or red, derive the next smallest roadmap-aligned
   slice from repo state, docs, and Linear
6. create or pick exactly one Linear issue for that slice
7. move that issue to `In Progress`
8. start the next branch and continue implementation

During that explicit continuation scope, the agent must not send a final merely
because one ticket, PR, archive, Linear deletion, or branch cleanup is complete.

Allowed stop conditions:

- missing permissions, authentication, required token, or unavailable external
  system access
- a destructive Git or filesystem operation would be required and has not been
  explicitly requested
- unrelated user changes block the current slice and cannot be worked around
  without risking data loss or undoing user work
- the next product decision is genuinely ambiguous and a reasonable assumption
  would create architecture, workflow, or musician-facing risk
- CI, tests, or local validation are failing and the cause cannot yet be
  classified after a reasonable investigation

Waiting for the required human verdict is an allowed stop condition; state the
exact review artifact and next human action.

When explicit phase-level or cross-ticket continuation is active, these are
non-stop conditions:

- the current ticket is merged, archived, deleted from Linear, or otherwise
  fully closed
- a bounded slice within an active phase is complete while the user has asked
  for phase completion or autonomous continuation
- a PR is open, CI is running, or CI is green for a locally clean branch
- local `main` is clean and synced
- Linear has been tidied or the current branch has been deleted
- the next slice needs to be derived from roadmap, specs, Linear, and repo state
- the agent has just given a status update or final-looking progress summary

If explicit cross-ticket continuation is active and no allowed stop condition
applies, continue the loop. Choose one small coherent next slice, keep Linear
state honest, and do not open multiple parallel tickets just to avoid stopping.

No Ticket, No Branch:

- before creating or switching to a feature branch for implementation, confirm
  the current slice has a Linear issue
- if no issue exists, create the issue first from roadmap/spec/repo context
- move that issue to `In Progress` before branch creation or reuse
- if branch work has already started without a Linear issue, stop expanding the
  code diff and repair Linear immediately before continuing
- do not treat later Linear cleanup as equivalent to issue-first execution;
  backfilled Done tickets are only damage repair, not a valid normal path

When the user asks to implement a full active phase, such as P013, completing a
single slice is only a handoff point inside the loop. Do the PR/CI/merge/archive
closeout for that slice, then derive and start the next phase-aligned slice
without sending a final status report unless an allowed stop condition applies.

### 2.1 Final Response Gate

Before sending a final response, the agent must ask:

`Is the requested bounded scope complete, or did the user explicitly request
phase-level or cross-ticket continuation?`

If the bounded scope is complete and no cross-ticket continuation was requested,
a concise final handoff is correct. If continuation was requested and no
allowed stop condition applies, continue the workflow state machine above.

A final response is allowed only when one of these is true:

- the user explicitly says `stop`, `pause`, or asks only for status,
  explanation, or discussion
- the requested bounded ticket/task is verified complete and the user did not
  request continuation into another ticket
- an allowed stop condition from this document applies
- the agent is blocked after reasonable investigation and can name the concrete
  blocker
- the user changes the task away from implementation work

This gate prevents both premature stopping inside an explicitly continued phase
and unrequested expansion from one completed ticket into another.

---

## 3. Normal Slice Flow

For a normal implementation or docs slice:

0. create or pick exactly one Linear issue for the slice; if the issue does not
   exist yet, create it before any branch work
1. move that Linear issue to `In Progress`
2. create or switch to a dedicated branch for that issue
3. make one coherent slice of changes
4. run the relevant local verification
5. run `code-review` on the branch diff when that skill is available
6. fix findings and answer review questions from that branch-level review
7. do a short self-review on the branch diff
8. open a PR
9. inspect GitHub Actions / CI output for the PR
10. if CI is red and the failure belongs to the slice, fix it before treating the review boundary as clean
11. move the Linear issue to `In Review`
12. add a human-readable issue update
13. add a project-level update in the `Riotbox Project Updates` Linear document
14. treat merge / approval as the boundary for closing this ticket; start the
    next Linear-first slice while CI runs only when explicit cross-ticket
    continuation is active
15. after merge, sync local `main`
16. move the issue to `Done`
17. archive useful Linear context before deletion when the ticket should be removed from Linear
18. after syncing `main`, delete the merged remote and local feature branches
    unless the branch is intentionally long-lived
19. delete the completed Linear issue only after the archive entry exists

The review handoff must be visible and evidence-backed. Before PR or closeout,
report the reviewed diff/scope, every finding with severity and concrete code or
contract evidence, and each finding's disposition (`fixed`, `deferred`, or
`rejected` with rationale). For a fixed finding, name the correction and the
test or inspection that verifies it. Then report the follow-up self-review
result, including an explicit zero-findings result when applicable. Do not
collapse this evidence into only “review passed” or “CI green.”

A prompt such as "weiter" continues the current requested scope. Starting
another ticket requires an explicit cross-ticket/phase instruction or a newly
requested issue. Such instructions change only whether another workflow
iteration runs; they never change steps 0-19 or permit work without an active
Linear issue.

### 3.1 Work Classification

Classify every P023 slice before implementation:

- `audible_vertical_slice`: changes the musician journey and must prove the
  exact live runtime/mixer path when it claims instrument progress
- `contract_enabler`: changes a prerequisite contract and names exactly one
  directly enabled audible follow-up issue and outcome
- `maintenance/regression`: preserves behavior and must not be presented as
  musical or instrument progress

Apply the [audio-QA contract](./specs/audio_qa_workflow_spec.md) to audible work.
Waiting for its required human verdict is an allowed stop condition; state the
exact review artifact and next human action.

For a new audible mechanism, `audible_vertical_slice` contains two ordered
stages from the [P023 audible-delivery plan](./plans/p023_audible_delivery_course_correction.md):
bounded `development_exploration`, then `product_qualification` only after a
provisional human keep. Exploration is not a fourth work class and cannot close
the ticket as product progress. Do not build complete product surfaces,
multi-source promotion packs, or a new validator framework merely to hear an
unkept idea.

---

## 4. Ambiguous Feedback Handling

Product feedback, later ideas, and immediate implementation requests can arrive
together.

When the intended next action is ambiguous, ask one concise clarifying question
before choosing a work lane. Do not silently turn open-ended feedback into
implementation work, and do not silently park an urgent implementation request
as a future note.

If the user labels something as later work, capture it as a ticket, repo note,
or roadmap item. Proceed beyond the current ticket only when the user explicitly
requests cross-ticket or phase continuation.

### 4.1 Plan Anchoring

When a new implementation plan becomes an accepted work direction, do not leave
it only as a standalone planning file. Keep the detailed plan under
`docs/plans/`, link it from `docs/README.md`, connect it to the relevant phase in
`docs/execution_roadmap.md`, tighten `docs/phase_definition_of_done.md` when it
changes phase completion criteria, and add a short decision-log entry when it
freezes a durable direction. Do not duplicate the whole plan across those files.

---

## 5. Choosing The Next Ticket

The next ticket should not be chosen ad hoc.

Use `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, the most
relevant active feature spec, and the real current repo state after the most
recent merge. Prefer the smallest coherent slice that closes the nearest real
gap in the current product path. Do not define a long chain of future tickets
while the current slice is unresolved, and do not open a second architecture,
UI path, or speculative side branch unless the roadmap explicitly calls for a
spike.

Review artifacts are point-in-time evidence, not the live backlog. Verify older
findings against current `main`, current Linear, newer bounded reviews, and exact
archived ticket history when needed before creating a ticket from them.

When explicit cross-ticket continuation remains active and no documented stop
condition applies, choose the next smallest coherent roadmap-aligned slice from
the active backlog or derive one from the roadmap, active specs, Linear, and
actual repo state.

For P023, select work in this order:

1. a waiting structured human review when a review-ready candidate exists
2. the next unblocked `audible_vertical_slice` on the active Golden Path
3. a `contract_enabler` only when it names the exact next audible issue it unlocks
4. maintenance / regression work only when current risk justifies delaying the audible path

Inside a selected audible slice, prefer the smallest exact Development
exploration that can falsify musician value before broad qualification. Keep
release/PR gates intact; change their order, not their strength.

Do not select deferred P016, P021, or P022 work unless the issue names the exact
P023 Golden Path blocker it removes. Continue one ticket at a time; do not open
multiple parallel tickets merely to avoid stopping.
