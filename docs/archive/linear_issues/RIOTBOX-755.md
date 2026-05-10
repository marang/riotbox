# `RIOTBOX-755` Optimize AGENTS.md for agent readability

- Ticket: `RIOTBOX-755`
- Title: `Optimize AGENTS.md for agent readability`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-755/optimize-agentsmd-for-agent-readability`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-755-optimize-agents-readability`
- Linear branch: `feature/riotbox-755-optimize-agentsmd-for-agent-readability`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#748 (https://github.com/marang/riotbox/pull/748)`
- Merge commit: `0968d865101f8f005f546b2c1a599dadfd3b2d66`
- Deleted from Linear: `2026-05-10`
- Verification: `git diff --check`; manual heading/bullet structure check with `rg -n '^## |^### ' AGENTS.md`; critical term preservation spot check for `ActionCommand`, `JamAppState`, audio QA, Linear deletion, MemPalace, sandbox requirements, `just ci`, `code-review`, and `review-codebase`; GitHub Rust CI success on PR #748.
- Docs touched: `AGENTS.md`
- Follow-ups: `None`

## Why This Ticket Existed

`AGENTS.md` had accumulated correct Riotbox rules, but important architecture, audio QA, CI, Linear, archive, branch-cleanup, MemPalace, and sandbox constraints were spread through long sections. That made it more expensive for coding agents to parse and apply the highest-priority instructions.

## What Shipped

- Reorganized `AGENTS.md` around a concise `Critical Rules` section.
- Grouped the remaining instructions into source-of-truth, architecture, `ActionCommand`, realtime/audio, QA, PR/CI, Linear, archive, branch cleanup, MemPalace, sandbox, commands, stack, and layout sections.
- Preserved existing Riotbox-specific constraints instead of changing project behavior.
- Clarified the old near-term build order as historical guidance and points agents to live roadmap docs first.

## Notes

- The local Markdown linter was not available, so Markdown structure was manually checked.
- The `riotbox-development` skill was inspected during this slice and left unchanged because it is already compact and prioritized.
