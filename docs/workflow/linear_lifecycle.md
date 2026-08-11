# Linear Lifecycle

Parent: [Riotbox Workflow Conventions](../workflow_conventions.md)

Use for ticket selection, issue state, updates, backlog, and the Linear/Codex
integration. The parent owns Linear-first order and continuation; archive,
deletion, and closeout procedure lives in [Archive And Cleanup](./archive_cleanup.md).

## Issue Lifecycle

Use these workflow states consistently:

- `In Progress` when active work starts
- `In Review` when the PR is open
- `Done` when the PR is merged
- `Canceled` when the issue is obsolete or superseded

Before creating or reusing a feature branch, create or pick exactly one Linear
issue for the slice and move it to `In Progress`. Later cleanup or a backfilled
Done ticket is not equivalent to issue-first execution. Follow the exact order
in the core [Normal Slice Flow](../workflow_conventions.md#3-normal-slice-flow).

For the current Riotbox Linear setup:

- old onboarding noise can be canceled or deleted
- completed issues should be cleaned up deliberately because the free tier has issue-count limits

## Linear Updates

Two update layers are expected.

### Issue-Level Update

Add a short, human-readable update on the Linear issue when:

- the ticket moves to `In Review`
- important findings change the recommendation
- the PR is merged

The issue update should say:

- what changed
- what was verified
- what remains bounded or open

### Project-Level Update

Also add a short entry to the `Riotbox Project Updates` Linear document when:

- a meaningful slice is opened for review
- a meaningful slice is merged
- a cross-ticket change affects the roadmap or working mode

This is the reviewable cross-ticket history.

## Parallel Workflow Lane

When delegation is available, workflow upkeep may run in parallel with
implementation instead of waiting until the end.

Preferred split for substantial slices:

- implementation lane:
  - code changes
  - tests
  - branch review
  - PR content
  - merge readiness
- workflow / ops lane:
  - Linear state transitions
  - issue comments
  - project update document entries
  - repo archive preparation and similar process obligations

Rules:

- treat the workflow / ops lane as real work, not optional cleanup
- keep code state, git state, Linear state, and archive readiness moving together
- implementation may continue on the main thread while a parallel workflow lane or subagent keeps Linear state, project updates, and archive obligations aligned
- the main coordinating agent still owns correctness, final review, and final integration
- delegation should reduce workflow drift, not hide responsibility for it

## Linear / Codex Integration

Use the Linear / Codex integration as an operational accelerator, not as a
product source of truth.

Good uses:

- pull the Linear issue, project, priority, acceptance criteria, and attachments
  into the working context before starting a branch
- keep issue state aligned with the workflow: `In Progress`, `In Review`,
  `Done`, `Canceled`
- attach PR links, branch links, and short human-readable issue updates
- maintain the near-term backlog horizon from roadmap, specs, and current repo state
- inspect Linear-visible diffs and PR metadata as a cross-check alongside `gh`
- delegate workflow / ops lane tasks such as issue comments, PR link updates,
  project update entries, archive preparation, and backlog hygiene when the
  integration can do them reliably

Limits:

- do not treat Linear / Codex output as canonical product truth unless the
  decision is mirrored into repo docs, specs, decision log, or Git history
- do not use Linear comments as the only record for architecture, replay,
  source-timing, Action Lexicon, audio QA, or musician-facing product contracts
- do not let delegated workflow updates hide responsibility from the main coordinating agent
- use `gh` for GitHub PR creation, PR status, CI/check summaries, and PR merges;
  Linear-visible PR data is supporting evidence, not the primary GitHub control plane

## Backlog Horizon

Linear should not hold only the current ticket.

Keep two horizons visible:

- active work:
  - `In Progress`
  - `In Review`
- near-term backlog:
  - the next plausible, already-shaped tickets

Recommended operating shape:

- 1 main ticket in progress
- 1-5 near-next tickets in backlog
- milestone-level placeholders for later work when useful

Rules:

- do not leave the working backlog empty if the next likely slices are already clear
- treat this as a standing workflow rule, not just a planning preference
- before creating the next branch, ensure Linear has:
  - exactly one active slice ticket moved to `In Progress`
  - 1-5 near-next tickets in backlog when the next likely slices are already clear
- after merging and closing one ticket, it is acceptable for there to be no
  `In Progress` ticket only for the short transition while selecting the next
  issue; the next branch must not be created until the next issue is `In Progress`
- do not over-decompose distant phases into many detailed tickets too early
- prefer a small, honest backlog over a large speculative ticket tree
- derive backlog tickets from the roadmap, active specs, and current repo state

### Explicit Cross-Ticket Continuation

Continuation beyond the requested ticket requires an explicit phase-level or
cross-ticket instruction and is governed by the parent
[Core Rule](../workflow_conventions.md#2-core-rule),
[Final Response Gate](../workflow_conventions.md#21-final-response-gate), and
[Choosing The Next Ticket](../workflow_conventions.md#5-choosing-the-next-ticket).
Backlog hygiene must not soften or duplicate those rules.

## Priority And Labels

Priority rule:

- `In Progress` / `In Review` -> `High (2)`
- honest near-next backlog -> `Medium (3)`
- distant work -> `Low (4)` or unset
- archive / repo-ops slices -> usually `Medium (3)` unless urgent

Label rule:

- keep labels orthogonal to projects:
  - projects answer phase
  - labels answer slice type
- keep the base label set small:
  - `workflow`
  - `archive`
  - `ux`
  - `benchmark`
  - `review-followup`

## Tool-Assisted And Manual Boundaries

Automatic or tool-assisted enough:

- issue state transitions
- issue comments
- project update document edits
- issue deletion through the token-backed `issueDelete` helper after archive handoff

Still manual or only partially automated:

- issue cleanup choices such as delete vs cancel vs archive
