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
5. do a self-review on the branch diff
6. open a PR
7. move the Linear issue to `In Review`
8. add a human-readable issue update
9. add a project-level update in the `Riotbox Project Updates` Linear document
10. wait for merge / approval boundary before continuing to the next ticket
11. after merge, sync local `main`
12. move the issue to `Done`

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
- prefer not to continue the next main ticket until the PR is merged
- small follow-up fixes on the same PR are fine
- do not silently bundle the next unrelated slice into the same PR

This keeps review history and Linear issue history aligned.

## 7.1 Self-Review Before PR

Before opening a PR, do a branch-local code review first.

The minimum self-review pass should check:

- correctness and failure paths
- drift against the active specs in `docs/`
- whether new behavior is adequately covered by tests
- whether docs or workflow notes need to move with the code

If that self-review finds a real issue, fix it before opening the PR when feasible instead of pushing known defects into review.

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
