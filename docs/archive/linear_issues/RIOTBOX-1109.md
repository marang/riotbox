# `RIOTBOX-1109` Wire automated musical fitness gate into Justfile and audio QA CI

- Ticket: `RIOTBOX-1109`
- Title: `Wire automated musical fitness gate into Justfile and audio QA CI`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1109/wire-automated-musical-fitness-gate-into-justfile-and-audio-qa-ci`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1109-automated-musical-fitness-ci`
- Linear branch: `feature/riotbox-1109-wire-automated-musical-fitness-gate-into-justfile-and-audio`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `Infra`, `benchmark`, `workflow`
- PR: `#1089 (https://github.com/marang/riotbox/pull/1089)`
- Merge commit: `14604c46731f5afed3b4f2355fe57343980f1a03`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1089; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Why

The validator only improves development quality if it becomes part of the normal audio QA loop instead of living as an optional script.

## Scope

Add repo commands that run the automated musical fitness fixtures and integrate the cheap deterministic gate into current audio QA CI.

## Required Commands

Add or update Justfile targets equivalent to:

* `automated-musical-fitness-fixtures`
* `automated-musical-fitness showcase=...`
* include the fixture-level gate in `audio-qa-ci`

## Boundaries

The CI gate should use deterministic fixtures and token-bounded logs. Real-source showcase packs may remain local/manual if they are too heavy or require generated artifacts that do not belong in CI.

## Acceptance

* `just audio-qa-ci` runs the deterministic automated musical fitness fixture gate.
* Local/manual commands exist for richer showcase or real-source review packs.
* Failure output identifies the fixture and failure code without dumping huge logs.
* The gate does not require audio hardware or realtime device access.

## What Shipped

- Closed the bounded scope: Wire automated musical fitness gate into Justfile and audio QA CI.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
