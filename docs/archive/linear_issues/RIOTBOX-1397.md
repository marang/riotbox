# `RIOTBOX-1397` P023: Reset active phase and operating rules around audible verticals

- Ticket: `RIOTBOX-1397`
- Title: `P023: Reset active phase and operating rules around audible verticals`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1397/p023-reset-active-phase-and-operating-rules-around-audible-verticals`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M1 | Reset + Real Listening`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-11`
- Finished: `2026-07-11`
- Branch: `feature/riotbox-1397-p023-reset-active-phase-and-operating-rules-around-audible`
- Linear branch: `feature/riotbox-1397-p023-reset-active-phase-and-operating-rules-around-audible`
- Assignee: `Markus`
- Labels: `Docs`, `Improvement`, `workflow`
- PR: `#1361 (https://github.com/marang/riotbox/pull/1361)`
- Merge commit: `a9553687f1df0e61b3b8e49d560a142942d9fb92`
- Deleted from Linear: `2026-07-11`
- Verification: `just check: passed; release-grade-demo-bank-fixtures: passed; both project skill quick validators: passed; git diff --check: passed; GitHub Actions Rust CI: passed`
- Docs touched: `docs/execution_roadmap.md; docs/phase_definition_of_done.md; docs/architecture_phase_map.md; docs/workflow_conventions.md; docs/specs/release_grade_musician_demo_bank_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1398 real human candidate verdicts; RIOTBOX-1403 fixture/live-readiness separation; RIOTBOX-1399 narrow PR validation; then RIOTBOX-1330/1333/1335 live Golden Path.`

## Why This Ticket Existed

Align canonical operating contracts around one live human-passed P023 Usable Musical Alpha instead of continued diagnostic activity.

## What Shipped

- Made P023 the single active product priority and deferred P016/P021/P022 unless they remove a named Golden Path blocker.
- Added audible vertical, contract enabler, and maintenance/regression work classes plus the exact-live-path and two-unverified-candidate listening stop rules.
- Corrected demo-bank semantics so fixture verdicts are not human coverage and weak/bad-timing sources may pass through reviewed degraded, unavailable, or reject behavior.

## Notes

- Operating-contract slice only; no audible sound improvement was claimed.
