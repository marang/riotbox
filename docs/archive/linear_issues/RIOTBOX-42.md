# `RIOTBOX-42` Ticket Archive

- Ticket: `RIOTBOX-42`
- Title: `Add first TR-909 source-support and takeover render profiles`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-42/add-first-tr-909-source-support-and-takeover-render-profiles`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-42-tr909-render-profiles`
- Linear branch: `feature/riotbox-42-add-first-tr-909-source-support-and-takeover-render-profiles`
- Assignee: `Markus`
- Labels: `None`
- PR: `#36`
- Merge commit: `0d7be6f`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-43`, `RIOTBOX-44`

## Why This Ticket Existed

`RIOTBOX-41` made TR-909 reinforcement audibly real, but the audible result was still too flat. Support and takeover had mode-level differences, yet there was no explicit profile layer telling the callback how a source-support context should differ from a takeover context.

## What Shipped

- made the TR-909 render contract carry typed source-support and takeover profiles
- derived source-support profiles from the current source section in `riotbox-app`
- derived takeover render profiles from committed TR-909 lane state instead of callback-side strings
- let the callback-side renderer vary subdivision, trigger density, envelope, and pitch from those profiles
- added regression tests for app-side profile derivation and callback-side audible profile differences

## Notes

- The slice stayed behind the existing app-to-audio render seam and did not reopen action, queue, or takeover control semantics.
- Branch review after the first green implementation found two bounded follow-up fixes before merge: one test expectation still assumed commit-boundary state instead of current transport state, and one callback-profile test needed a stronger activity metric than peak amplitude. Both were fixed on the branch before the PR was considered clean.
- Merge used local green verification plus a mergeable PR because the GitHub connector again reported an empty external status set in this environment.
