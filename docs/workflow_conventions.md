# Riotbox Workflow Conventions

Version: 0.1  
Status: Draft  
Audience: contributors, reviewers, coding agents

---

## 1. Purpose

This document records the current working GitHub and Linear convention for Riotbox.

It exists so work stays:

- reviewable
- ticket-linked
- easy to reconstruct later
- consistent across code, docs, and planning slices

This is a workflow note, not a product spec.

---

## 2. Core Rule

Default workflow:

`Linear issue -> branch -> scoped commit(s) -> PR -> review -> merge -> sync local main`

Do not skip the PR step for normal feature or implementation work.

---

## 3. Normal Slice Flow

For a normal implementation or docs slice:

1. move the Linear issue to `In Progress`
2. create a dedicated branch for that issue
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
14. wait for merge / approval boundary before continuing to the next ticket
15. after merge, sync local `main`
16. move the issue to `Done`

This is the default unless the user explicitly asks for something else.

---

## 4. Branch Naming

Preferred branch pattern:

- `riotbox-<issue-number>-<short-slice-name>`

Examples:

- `riotbox-18-analysis-ingest-slice`
- `riotbox-19-decoded-source-baseline`

Rules:

- keep the name short and human-readable
- keep one branch aligned to one main issue
- do not overload a branch with unrelated slices

Linear may suggest its own branch names. That is useful context, but the repo branch should stay concise and readable.

---

## 5. Commit Scope

Preferred commit style:

- one coherent slice per commit where possible
- commit message should describe the slice outcome, not just the file touched

Good examples:

- `add first analysis ingest slice`
- `operationalize mempalace dev tooling`
- `document PR description guideline`

Avoid mixing unrelated cleanup into the same commit unless it is required for the slice to pass.

---

## 6. Pull Request Rules

Every completed ticket should normally open a PR.

PR descriptions should include:

- `Why This Matters`
- `Summary`
- `Verification`

`Why This Matters` must explain:

- what larger phase or milestone the slice belongs to
- what product path or architecture seam it unlocks
- what remains intentionally bounded, stubbed, or out of scope

Do not write PR descriptions as changelogs only.

---

## 7. Review Boundary

Once a PR is open for a ticket:

- treat that ticket as being at the review boundary
- inspect the CI / GitHub Actions output explicitly
- if CI is red, treat the branch as still active work until the relevant failures are addressed
- prefer not to continue the next main ticket until the PR is merged
- small follow-up fixes on the same PR are fine
- do not silently bundle the next unrelated slice into the same PR

This keeps review history and Linear issue history aligned.

## 7.1 Branch-Level Review Before PR

Before opening a PR for a finished feature branch:

- run the `code-review` skill when it is available in the current session
- use that review to surface findings, fix them on the same branch, and answer review questions before the PR is created
- then do the normal short self-review pass as a final check

Minimum branch-level review expectations:

- correctness and failure paths
- drift against the active specs in `docs/`
- whether new behavior is adequately covered by tests
- whether docs or workflow notes need to move with the code

If the `code-review` skill is not available in the current session:

- state that clearly in the working notes or PR context
- fall back to the normal self-review pass instead of skipping review entirely

## 7.2 CI Check After PR Open

After opening a PR, explicitly inspect the GitHub Actions / CI status.

Minimum expectation:

- formatter check passes
- test suite passes
- lint / static analysis passes
- any slice-specific workflow required by the repo is checked

Rules:

- do not assume CI is fine just because local checks passed
- if a CI failure is caused by the current slice, fix it on the same branch before treating the ticket as cleanly in review
- mention important CI failures and fixes in the Linear issue update when they happen

## 7.3 Periodic Whole-Codebase Review

Branch-level review is not enough on its own.

On a regular cadence, run the `review-codebase` skill for a broader whole-repo review.

Default cadence:

- after every 5th finished feature branch

Purpose:

- catch cross-slice architecture drift
- detect recurring correctness or testing gaps
- find patterns that do not show up clearly in one branch diff

Expected outputs:

- a review artifact under `docs/reviews/`
- important resulting decisions or constraints in `docs/research_decision_log.md`
- workflow or spec updates if the review changes how the repo should be operated

If the `review-codebase` skill is not available in the current session, fall back to a manual whole-codebase review and record that fact explicitly.

---

## 8. Linear Updates

Two update layers are expected:

### 8.1 Issue-level update

Add a short, human-readable update on the Linear issue when:

- the ticket moves to `In Review`
- important findings change the recommendation
- the PR is merged

The issue update should say:

- what changed
- what was verified
- what remains bounded or open

### 8.2 Project-level update

Also add a short entry to the `Riotbox Project Updates` Linear document when:

- a meaningful slice is opened for review
- a meaningful slice is merged
- a cross-ticket change affects the roadmap or working mode

This is the reviewable cross-ticket history.

## 8.3 Backlog Horizon

Linear should not hold only the current ticket.

Keep two horizons visible:

- active work:
  - `In Progress`
  - `In Review`
- near-term backlog:
  - the next plausible, already-shaped tickets

Recommended operating shape:

- 1 main ticket in progress
- 1-3 near-next tickets in backlog
- milestone-level placeholders for later work when useful

Rules:

- do not leave backlog empty if the next likely slices are already clear
- do not over-decompose distant phases into many detailed tickets too early
- prefer a small, honest backlog over a large speculative ticket tree
- derive backlog tickets from the roadmap, active specs, and current repo state

## 8.4 Retention And Cleanup

Linear is the active execution surface, not the canonical long-term archive.

Because the workspace runs on the free tier, completed issues should not accumulate forever.

Use this retention model:

- Linear:
  - active ticket flow
  - near-term backlog
  - recent completed work while it is still operationally useful
- repo archive:
  - long-term ticket history worth keeping
- MemPalace:
  - retrieval layer over repo docs and archive material, not canonical storage

Before deleting a completed Linear issue, preserve its useful context in repo markdown under `docs/archive/linear_issues/`.

Recommended archive shapes:

- one file per ticket for architecture, review, decision, or process tickets
- grouped monthly or phase-oriented files for routine feature tickets when readability stays good

Each archived ticket entry should include at least:

- ticket id and title
- phase or milestone
- status date or merge point
- why the ticket existed
- what shipped
- PR link
- merge commit
- follow-up tickets or bounded open questions

Deletion rule:

- do not delete a Linear ticket until:
  - the PR is merged
  - the issue is marked done
  - the repo archive entry exists

---

## 9. Automatic vs Manual Behavior

Current practical split:

### Automatic or tool-assisted enough

- local branch creation
- commit and push flow
- PR creation
- issue state transitions
- issue comments
- project update document edits

### Still manual or only partially automated

- final judgment about whether a slice is ready for review
- any direct PR description edits if the available connector/tooling cannot patch the body later
- issue cleanup choices such as delete vs cancel vs archive

Because of that, the safe rule is:

- make the PR description correct at creation time
- do not rely on later cleanup if it can be avoided

---

## 10. Direct Push To `main`

Normal slice work should not go directly to `main`.

Direct push to `main` is acceptable only when all of the following are true:

- the user explicitly asked for it
- the change is very small
- the change is repo-meta or workflow-meta rather than product implementation
- skipping the PR does not hide meaningful review risk

Examples of acceptable direct-to-`main` exceptions:

- a tiny `AGENTS.md` rule update
- a very small repo convention note

Examples that should still use a PR:

- code changes
- architectural changes
- anything that advances a real product slice

---

## 11. Local Sync After Merge

After a PR is merged:

1. switch back to `main`
2. fetch `origin`
3. fast-forward local `main`

Do not continue new ticket work on stale local `main`.

---

## 12. Issue Lifecycle Notes

Use these workflow states consistently:

- `In Progress` when active work starts
- `In Review` when the PR is open
- `Done` when the PR is merged
- `Canceled` when the issue is obsolete or superseded

For the current Riotbox Linear setup:

- old onboarding noise can be canceled or deleted
- completed issues should be cleaned up deliberately because the free tier has issue-count limits

---

## 13. Current Standing Exceptions

Current known practical exceptions:

- project-level chronological updates currently live in the `Riotbox Project Updates` Linear document
- MemPalace is optional workflow tooling, not canonical process state
- tiny workflow-note changes may still be pushed directly to `main` if explicitly approved by the user

---

## 14. Choosing The Next Ticket

The next ticket should not be chosen ad hoc.

Default decision inputs:

- `docs/execution_roadmap.md`
- `docs/phase_definition_of_done.md`
- the most relevant active feature spec
- the real current repo state after the most recent merge

Decision rule:

- prefer the smallest coherent slice that closes the nearest real gap in the current product path
- do not define a long chain of future tickets in full detail while the current slice is still unresolved
- avoid choosing tickets that open a second architecture, second UI path, or speculative side branch unless the roadmap explicitly calls for a spike

Useful check questions:

1. what phase are we in?
2. what is the sharpest missing capability or blocker right now?
3. what is the smallest slice that moves that capability forward?
4. does this ticket keep Riotbox on the documented product spine?
5. will the result be easy to review as one coherent PR?

---

## 15. Short Version

If unsure, do this:

1. pick one Linear ticket
2. create one branch
3. make one coherent slice
4. verify locally
5. open one PR with `Why This Matters`
6. update the issue and project log
7. wait for merge
8. sync `main`
