# `RIOTBOX-1498` Make Sidecar timeout regression independent of Python startup scheduling

- Ticket: `RIOTBOX-1498`
- Title: `Make Sidecar timeout regression independent of Python startup scheduling`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1498/make-sidecar-timeout-regression-independent-of-python-startup`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-22`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1498-sidecar-timeout-test`
- Linear branch: `feature/riotbox-1498-make-sidecar-timeout-regression-independent-of-python`
- Assignee: `Markus`
- Labels: `Bug`, `Infra`
- PR: `#1529 (https://github.com/marang/riotbox/pull/1529)`
- Merge commit: `cebd5b7459b7772ea5babd616b25aa626b74f24a`
- Deleted from Linear: `2026-09-22`
- Verification: `Full normal-parallel local just ci and GitHub Rust CI passed; all 14 Sidecar tests and two current-executable 96-run/32-worker stress loops passed.`
- Docs touched: `docs/reviews/riotbox_1498_sidecar_timeout_test_2026-09-22.md`
- Follow-ups: `None`

## Why This Ticket Existed

A short operation-budget regression accidentally timed Python process startup, intermittently interrupting parallel CI.

## What Shipped

- Separate bounded fixture readiness from operation-budget testing and add deterministic delayed startup; production timeouts unchanged.

## Notes

- Pre-fix stress failed 96/96 and 93/96; deterministic delayed-start regression failed before the test correction. No production/audio change.
