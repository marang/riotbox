# `RIOTBOX-1382` Repo: Add CodeRabbit review configuration

- Ticket: `RIOTBOX-1382`
- Title: `Repo: Add CodeRabbit review configuration`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1382/repo-add-coderabbit-review-configuration`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1382-repo-add-coderabbit-review-configuration`
- Linear branch: `feature/riotbox-1382-repo-add-coderabbit-review-configuration`
- Assignee: `Markus`
- Labels: None
- PR: `#1346 (https://github.com/marang/riotbox/pull/1346)`
- Merge commit: `87252c357ebf0971ead336d1e75c6d2b272e69f5`
- Deleted from Linear: `2026-07-05`
- Verification: `PyYAML parsed .coderabbit.yaml; git diff --check; GitHub rust-ci passed on PR #1346`
- Docs touched: `.coderabbit.yaml; docs/workflow_conventions.md`
- Follow-ups: `Repository or organization owner must install/authorize the CodeRabbit GitHub App for marang/riotbox if it is not already installed.`

## Why This Ticket Existed

Add CodeRabbit as an advisory automated PR reviewer without making it a merge gate or weakening the Linear-first workflow.

## What Shipped

- Added repo CodeRabbit config and documented that CodeRabbit comments must be re-checked, then fixed or ticketed when relevant, while implementation can continue when CI is clean.

## Notes

- No audio-producing behavior changed.
