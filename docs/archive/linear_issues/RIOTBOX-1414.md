# `RIOTBOX-1414` Tighten app/core layering and committed side-effect dispatch

- Ticket: `RIOTBOX-1414`
- Title: `Tighten app/core layering and committed side-effect dispatch`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1414/tighten-appcore-layering-and-committed-side-effect-dispatch`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1414-commit-ownership`
- Linear branch: `feature/riotbox-1414-tighten-appcore-layering-and-committed-side-effect-dispatch`
- Assignee: `Markus`
- Labels: `Core`, `Improvement`, `review-followup`
- PR: `#1535 (https://github.com/marang/riotbox/pull/1535)`
- Merge commit: `5aceddc72f28cc59ed8bc9fa598f74bb18d4425e`
- Deleted from Linear: `2026-09-22`
- Verification: `3840 differential state/result comparisons and added compatibility/order tests pass. Full integrated normal-parallel local just ci, strict Clippy, include guard and GitHub Rust CI passed.`
- Docs touched: `docs/specs/action_lexicon_spec.md,docs/specs/session_file_spec.md,docs/reviews/riotbox_1414_commit_ownership_2026-09-22.md`
- Follow-ups: `None`

## Why This Ticket Existed

Broadcast side-effect dispatch, policy imports from the view and duplicate boundary ranks obscured ownership.

## What Shipped

- Exhaustive 64-command dispatch; Session-owned timing confirmation with compatible exports; one boundary rank for quantization and replay. Existing handler algorithms and ordering preserved.

## Notes

- No new action registry, wire shape, DSP, real-source access or human playback. Solo Rust branch review and self-review: no retained findings.
