# `RIOTBOX-1115` Move Riotbox Codex skills into project-local .codex tree

- Ticket: `RIOTBOX-1115`
- Title: `Move Riotbox Codex skills into project-local .codex tree`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1115/move-riotbox-codex-skills-into-project-local-codex-tree`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1115-move-riotbox-codex-skills-into-project-local-codex-tree`
- Linear branch: `feature/riotbox-1115-move-riotbox-codex-skills-into-project-local-codex-tree`
- Assignee: `Markus`
- Labels: `Docs`, `workflow`
- PR: `#1093 (https://github.com/marang/riotbox/pull/1093)`
- Merge commit: `a0934f2b4d2587dcdbb5964647c8f2316cdd34f6`
- Deleted from Linear: `2026-06-03`
- Verification: `local: scripts/run_compact.sh /tmp/riotbox-1115-just-ci.log just ci -> pass`; `GitHub Actions: rust-ci on PR #1093 -> pass`
- Docs touched: `AGENTS.md`, `.codex/skills/riotbox-development/SKILL.md`, `.codex/skills/riotbox-rave-punk-production/SKILL.md`, `.codex/skills/riotbox-listening-review/SKILL.md`
- Follow-ups: `None`

## Why This Ticket Existed

Riotbox Codex skills were still machine-local or represented only by an empty .codex placeholder, which made agent behavior easy to drift outside Git history. The project needed repo-owned skills so future agents load the same audio QA, product-taste, workflow, and structured listening-review guidance.

## What Shipped

- Replaced the empty tracked .codex placeholder with project-owned .codex/skills.
- Added riotbox-development and riotbox-rave-punk-production as repo-local skills.
- Added riotbox-listening-review as the linked RIOTBOX-1114 companion skill for structured human listening review.
- Updated AGENTS.md to make project-local skills canonical and home-directory skill copies a legacy fallback only.

## Notes

- No audible behavior changed; human_verdict/listening-review gates were not applicable to this repo-ops slice.
