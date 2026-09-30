# `RIOTBOX-1504` Bound Sidecar request writes by the operation deadline

- Ticket: `RIOTBOX-1504`
- Title: `Bound Sidecar request writes by the operation deadline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1504/bound-sidecar-request-writes-by-the-operation-deadline`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1504-sidecar-transport-deadline`
- Linear branch: `feature/riotbox-1504-bound-sidecar-request-writes-by-the-operation-deadline`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#1547 (https://github.com/marang/riotbox/pull/1547)`
- Merge commit: `bc819b0d389622b1c07a1494632cdcceb80ff6af`
- Deleted from Linear: `2026-09-30`
- Verification: `Local just ci passed: App 770, Audio 279, Core 470, Sidecar 24. GitHub run 36760004684 passed Ubuntu and Windows Sidecar tests/strict Clippy for a3e268c88182eb798c6ca0a1372f9873f5ec2fed; PR reviewed solo and merged bc819b0d389622b1c07a1494632cdcceb80ff6af.`
- Docs touched: `RBX-382, technology_stack_spec.md, riotbox_1504_sidecar_transport_deadline_2026-09-30.md`
- Follow-ups: `None`

## Why This Ticket Existed

Offline Sidecar calls could hang on request pipe backpressure before their response timeout began.

## What Shipped

- One shared pipe deadline; semantic synchronous transport and platform modules; close/reap invalid peers; source-free regressions and native Windows CI.

## Notes

- Maintenance/regression, source-free only. Initial Windows fixture failures corrected without production path/framing change; native rerun green. No source/holdout, human listening, DAW or Windows audio qualification. No independent reviewer approval.
