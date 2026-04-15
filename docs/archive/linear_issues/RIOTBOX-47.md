# `RIOTBOX-47` Ticket Archive

- Ticket: `RIOTBOX-47`
- Title: `Add fixture-backed TR-909 pattern-adoption regression coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-47/add-fixture-backed-tr-909-pattern-adoption-regression-coverage`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-47-pattern-adoption-fixtures`
- Linear branch: `feature/riotbox-47-add-fixture-backed-tr-909-pattern-adoption-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#41`
- Merge commit: `444f795`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-48`, `RIOTBOX-49`, `RIOTBOX-50`

## Why This Ticket Existed

`RIOTBOX-45` and `RIOTBOX-46` deepened the audible TR-909 render seam with pattern adoption and phrase variation, but the fixture coverage was still too narrow around `fills`, `break_reinforce`, and `scene_lock` takeover cases. The next bounded slice needed to widen replay-safe verification without introducing any new device behavior or a second approval path.

## What Shipped

- expanded the app-side committed render projection fixture set with additional `scene_lock`, `fills`, and `break_reinforce` cases
- expanded the audio-side callback regression fixture set across the same TR-909 render modes
- recalibrated one `scene_lock` expectation against the real callback output instead of guessing ranges
- kept the slice verification-only with no new queue, render, or shell behavior

## Notes

- The ticket intentionally changed only fixture coverage; the shipped TR-909 behavior stayed exactly where it already was.
- GitHub's combined-status endpoint again returned an empty status set in this environment, and `gh pr checks` was unavailable because the local CLI was not authenticated, so merge used explicit local green verification plus a mergeable PR.
