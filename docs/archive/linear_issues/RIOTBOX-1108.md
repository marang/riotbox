# `RIOTBOX-1108` Add automated musical fitness positive and negative fixture corpus

- Ticket: `RIOTBOX-1108`
- Title: `Add automated musical fitness positive and negative fixture corpus`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1108/add-automated-musical-fitness-positive-and-negative-fixture-corpus`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1108-automated-musical-fitness-fixtures`
- Linear branch: `feature/riotbox-1108-add-automated-musical-fitness-positive-and-negative-fixture`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `benchmark`
- PR: `#1088 (https://github.com/marang/riotbox/pull/1088)`
- Merge commit: `3fd17ab83470b8f12513d54b0b539b65e65acebc`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1088; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Why

The musical fitness gate needs deterministic fixtures that prove it rejects broken output modes instead of only scoring happy-path showcase artifacts.

## Scope

Add a small fixture corpus for the automated musical fitness validator. Fixtures should be cheap, deterministic, and safe for CI.

## Fixture Families

Create or generate fixtures for:

* valid source-reactive pack
* invalid static loop
* invalid source-masked output
* invalid weak low-end / weak transient output
* invalid identical response across different sources
* invalid fallback-collapsed output
* invalid grid-drift output

## Acceptance

* Fixture generation/validation is reproducible from repo commands.
* Each negative fixture fails for the intended reason.
* The valid fixture passes without relying on human listening.
* Fixture names and reports make it obvious that this is automated fitness, not human taste approval.

## What Shipped

- Closed the bounded scope: Add automated musical fitness positive and negative fixture corpus.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
