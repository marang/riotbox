# `RIOTBOX-1074` Enforce stem non-silence in QA gate when artifact metrics exist

- Ticket: `RIOTBOX-1074`
- Title: `Enforce stem non-silence in QA gate when artifact metrics exist`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1074/enforce-stem-non-silence-in-qa-gate-when-artifact-metrics-exist`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1074-stem-non-silence-gate`
- Linear branch: `feature/riotbox-1074-enforce-stem-non-silence-in-qa-gate-when-artifact-metrics`
- Assignee: `Markus`
- Labels: None
- PR: `#1050 (https://github.com/marang/riotbox/pull/1050)`
- Merge commit: `b053e29deae10ea6f5f60a38f9db53eb5d98a1ea`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core stem_package -- --nocapture`; `cargo test -p riotbox-core audio_metrics -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1050`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1075`

## Why This Ticket Existed

Per-artifact audio metrics now exist, so the stem QA skeleton can fail obvious silence without claiming full stem export readiness.

## What Shipped

- Enforced non-silence when claimed stem artifacts include metrics.
- Failed metrics that prove silence or cannot prove activity.
- Kept missing metrics and fallback-collapse as deferred checks where evidence is still absent.
- Updated Audio QA workflow spec with the current gate boundary.

## Notes

- No stem files, stem package export action, or fallback-collapse claim shipped.
