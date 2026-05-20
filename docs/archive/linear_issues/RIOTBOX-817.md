# `RIOTBOX-817` Clamp Source Timing probe evidence scores in JSON validator

- Ticket: `RIOTBOX-817`
- Title: `Clamp Source Timing probe evidence scores in JSON validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-817/clamp-source-timing-probe-evidence-scores-in-json-validator`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-817-source-timing-score-validator`
- Linear branch: `feature/riotbox-817-clamp-source-timing-probe-evidence-scores-in-json-validator`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#812 (https://github.com/marang/riotbox/pull/812)`
- Merge commit: `59e7852b9d94fa1822886853ed54e2b8796c87b8`
- Verification: `GitHub Actions Rust CI #1975 passed; local just ci passed; source-timing probe JSON validator fixtures passed; source-timing example report fixtures passed.`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None from this slice.`

## Why This Ticket Existed

After RIOTBOX-816 made readiness evidence visible, probe score and ratio fields needed validator bounds so impossible values could not pass QA.

## What Shipped

- Clamped Source Timing probe evidence scores and ratios to 0..1 or null in the JSON validator, added an invalid score-range fixture, wired it into the validation gate, and documented the bounded contract.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.
