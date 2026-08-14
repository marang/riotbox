# `RIOTBOX-1413` Carry MC-202 role as a typed action parameter through queue, commit, and replay

- Ticket: `RIOTBOX-1413`
- Title: `Carry MC-202 role as a typed action parameter through queue, commit, and replay`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1413/carry-mc-202-role-as-a-typed-action-parameter-through-queue-commit-and`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Canceled`
- Created: `2026-07-19`
- Started: `Unknown`
- Finished: `2026-07-19`
- Branch: `feature/riotbox-1413-carry-mc-202-role-as-a-typed-action-parameter-through-queue`
- Linear branch: `feature/riotbox-1413-carry-mc-202-role-as-a-typed-action-parameter-through-queue`
- Assignee: `Unassigned`
- Labels: `Core`, `Improvement`, `review-followup`
- PR: None
- Merge commit: `None`
- Deleted from Linear: `2026-08-14`
- Verification: `Terminal Linear state and repository history verified during historical cleanup.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Freshness classification

Canceled as an intentionally deferred review candidate, not an open implementation defect.

The broad review correctly observed that `Mc202SetRole` still crosses the persisted action boundary through stable role labels. However, this exact seam was already investigated in RIOTBOX-747 through RIOTBOX-751:

* typed role/phrase helpers are already used by behavior consumers
* unknown labels already reject explicitly at the typed boundary
* the accepted Session v1 decision intentionally preserves stable compatibility labels in persisted actions/session/undo evidence
* a wire-shape migration is allowed only as part of a documented broader Session version migration

Canonical local evidence:

* `docs/reviews/mc202_typed_contract_migration_plan_2026-05-10.md`
* `docs/archive/linear_issues/RIOTBOX-751.md`
* accepted decision at `docs/research_decision_log.md:2037-2043`

Creating a standalone typed ActionParams migration now would reopen a settled compatibility decision without a musician-facing benefit. Revisit only when Session vNext or genuinely new MC-202 role semantics require the wire migration.

## What Shipped

- No implementation shipped; the ticket was canceled before closeout and its rationale is preserved here.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
