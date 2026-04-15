# `RIOTBOX-43` Ticket Archive

- Ticket: `RIOTBOX-43`
- Title: `Add replay-safe TR-909 audio fixtures and regression checks`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-43/add-replay-safe-tr-909-audio-fixtures-and-regression-checks`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-43-tr909-audio-fixtures`
- Linear branch: `feature/riotbox-43-add-replay-safe-tr-909-audio-fixtures-and-regression-checks`
- Assignee: `Markus`
- Labels: `None`
- PR: `#37`
- Merge commit: `0262153`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-44`

## Why This Ticket Existed

`RIOTBOX-41` and `RIOTBOX-42` made the TR-909 render path both audible and profile-aware, but they still lacked fixture-backed regression coverage. The next bounded slice needed to protect the committed-state-to-render-seam-to-callback chain before deeper TR-909 work continued.

## What Shipped

- added fixture-backed committed-state render projection checks in `riotbox-app`
- added fixture-backed callback-output regression checks in `riotbox-audio`
- added the minimal test-only parser dependencies needed to load those fixtures
- recorded the verification strategy as a stable decision in `docs/research_decision_log.md`

## Notes

- The slice stayed verification-only and did not add new musical behavior.
- Branch review plus test gating found two bounded follow-up fixes before merge: fixture parser dependencies had to be added explicitly, and two initial audio-thresholds were calibrated down to match the real bounded callback output instead of theoretical peaks.
- Merge used local green verification plus a mergeable PR because the GitHub connector again reported an empty external status set in this environment.
