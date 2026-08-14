# `RIOTBOX-1111` Workflow: Remove MemPalace dev-memory tooling

- Ticket: `RIOTBOX-1111`
- Title: `Workflow: Remove MemPalace dev-memory tooling`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1111/workflow-remove-mempalace-dev-memory-tooling`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1111-remove-mempalace-tooling`
- Linear branch: `feature/riotbox-1111-workflow-remove-mempalace-dev-memory-tooling`
- Assignee: `Markus`
- Labels: None
- PR: `#1086 (https://github.com/marang/riotbox/pull/1086)`
- Merge commit: `0e6865434237abd42d43b7f0b213bc7d15a118d4`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1086; terminal Linear state and repository history verified during cleanup.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Goal:

Remove MemPalace from the Riotbox workflow because recent actual usage is near zero and the tool is not earning its maintenance cost.

Acceptance:

* Remove MemPalace wrapper scripts, container/compose files, and `just` commands.
* Update AGENTS.md and workflow/dev docs to stop instructing agents to use MemPalace.
* Keep canonical memory guidance focused on repo docs, Linear, Git, targeted `rg`, and bounded file reads.
* Preserve historical evaluation/archive docs as history unless they create active workflow instructions.
* Update decision/search helpers to use deterministic repo-local fallback only, or remove MemPalace dependency from them.
* Run relevant checks for command/docs consistency.

Why it matters:

A workflow tool that agents rarely use creates setup and maintenance drag without improving delivery. Removing MemPalace keeps Riotbox source-of-truth rules simpler and avoids implying there is a hidden memory layer agents must consult.

## What Shipped

- Removed MemPalace wrapper scripts, container files, commands, and active workflow guidance.
- Kept canonical project memory in repo documentation, Linear, Git, and bounded deterministic search.

## Notes

- Historical cleanup only; no product or audio behavior changed.
