# `RIOTBOX-142` Restore and simplify workflow reminder sidecar

- Ticket: `RIOTBOX-142`
- Title: `Restore and simplify workflow reminder sidecar`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-142/restore-and-simplify-workflow-reminder-sidecar`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-142-workflow-reminder-simplify`
- Linear branch: `feature/riotbox-142-restore-and-simplify-workflow-reminder-sidecar`
- Assignee: `Markus`
- Labels: `None`
- PR: `#134`
- Merge commit: `b226f7f70a9bd2a6087f4e2ef06f5d37eb886c92`
- Deleted from Linear: `Not deleted`
- Verification: `bash -n scripts/workflow_reminder_loop.sh scripts/start_workflow_reminder_tmux.sh scripts/check_workflow_reminder_tmux.sh`, `git diff --check`, manual tmux reminder session check, GitHub Actions `Rust CI` run `#363`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`
- Follow-ups: `RIOTBOX-143`

## Why This Ticket Existed

The earlier workflow reminder sidecar had been shelved while finishing active implementation slices, but the repo still needed one lightweight background nudge to keep autonomous runs moving and to reduce idle drift at clean checkpoints. The reminder needed to come back without the heavier log files, extra feed plumbing, or noisy sidecar overhead from the first attempt.

## What Shipped

- restored the workflow reminder loop as a single direct tmux-sidecar path
- slowed the reminder interval to a quiet 30-second cadence
- removed `CONVO_FEED` and `reports/subagents/*.log` churn from the reminder setup
- added one check script that verifies the tmux session and captures the latest reminder lines from pane history
- documented the reminder sidecar expectations in `AGENTS.md` and `docs/workflow_conventions.md`

## Notes

- this was a repo-ops slice only; it did not change any user-facing Riotbox product behavior
- the reminder is intentionally polling and lightweight, not an event-driven PR watcher
