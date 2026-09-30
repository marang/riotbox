# `RIOTBOX-1506` Warm Sidecar readiness before the short analysis-timeout regression

- Ticket: `RIOTBOX-1506`
- Title: `Warm Sidecar readiness before the short analysis-timeout regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1506/warm-sidecar-readiness-before-the-short-analysis-timeout-regression`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1506-sidecar-test-readiness`
- Linear branch: `feature/riotbox-1506-warm-sidecar-readiness-before-the-short-analysis-timeout`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`
- PR: `#1550 (https://github.com/marang/riotbox/pull/1550)`
- Merge commit: `800f1dfee708d980990af1b3cec26217193f64a0`
- Deleted from Linear: `2026-09-30`
- Verification: `One exact regression fails before correction with Control instead of Analysis. Sidecar 24 tests, CWD integration, strict Clippy and full local just ci pass. GitHub run 36763992585 Ubuntu and native Windows checks pass for 134bd813ffaca5295b37d004933826e9558536a3; PR #1550 merged.`
- Docs touched: `technology_stack_spec.md; riotbox_1506_sidecar_test_readiness_2026-09-30.md`
- Follow-ups: `None`

## Why This Ticket Existed

A short analysis-timeout regression could fail during cold Python readiness instead of testing analysis.

## What Shipped

- Warm readiness under default bounded policy; deterministic synthetic startup delay; unchanged production budgets; documented test convention.

## Notes

- Test-only maintenance; no production transport change, source/holdout, DAW, device or human listening. Sequential solo review, no independent approval.
