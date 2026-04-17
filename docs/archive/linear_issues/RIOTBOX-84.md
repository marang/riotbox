# `RIOTBOX-84` Scope W-30 operation diagnostics to the current lane target

- Ticket: `RIOTBOX-84`
- Title: `Scope W-30 operation diagnostics to the current lane target`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-84/scope-w-30-operation-diagnostics-to-the-current-lane-target`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-84-w30-diagnostics-lane-target`
- Linear branch: `feature/riotbox-84-scope-w-30-operation-diagnostics-to-the-current-lane-target`
- Assignee: `Markus`
- Labels: `None`
- PR: `#78`
- Merge commit: `5411424ff7ba1c3cdcc061430f7481c0c66c28e0`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#215`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-85`

## Why This Ticket Existed

The follow-up review found that the W-30 shell still derived bank-manager, damage-profile, and loop-freeze summaries from the most recent committed action of each command anywhere in history. After focus moved, `Jam`, `Capture`, and `Log` could still describe stale operations from another bank or pad. The same review also found a duplicated `latest promoted` line wasting scarce shell space in the lineage-active Capture branch.

## What Shipped

- scoped W-30 bank-swap, damage-profile, and loop-freeze summary lookups to the current lane target instead of global command history
- kept the pending-cue path unchanged while making committed summaries follow the current bank and pad focus
- removed the duplicate `latest promoted` line from the lineage-active Capture branch
- added shell regressions for current-target diagnostics and the no-duplicate lineage case

## Notes

- this slice changes no W-30 runtime or audio behavior; it only makes the current shell seam more trustworthy
- the next bounded W-30 follow-up is the first slice-pool browse control on the current lineage seam
