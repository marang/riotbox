# `RIOTBOX-41` Ticket Archive

- Ticket: `RIOTBOX-41`
- Title: `Make TR-909 reinforcement audibly real from the render seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-41/make-tr-909-reinforcement-audibly-real-from-the-render-seam`
- Project: `P005 | TR-909 MVP`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-41-audible-tr909-reinforcement`
- Linear branch: `feature/riotbox-41-make-tr-909-reinforcement-audibly-real-from-the-render-seam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#35`
- Merge commit: `11a7cdd`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-42`, `RIOTBOX-43`, `RIOTBOX-44`

## Why This Ticket Existed

`RIOTBOX-40` established an explicit TR-909 render seam, but Riotbox still did not produce any actual audible reinforcement from it. The next honest TR-909 MVP slice had to make the existing committed lane state audible on the real audio/runtime path without inventing a second hidden drum subsystem.

## What Shipped

- started the audio runtime from the app binary and kept the TUI alive even when audio startup fails
- extended the TR-909 render seam with tempo and beat-position data needed for callback-side rendering
- rendered bounded audible TR-909 reinforcement directly inside `riotbox-audio` from the existing `Tr909RenderState`
- updated app runtime state so the current committed TR-909 render projection continuously feeds the audio runtime
- added regression coverage for callback-side shared render-state updates, silence when idle, audible output in support mode, and exact silence when the drum-bus level is zero

## Notes

- The slice stayed intentionally bounded: it does not claim to be a full drum-machine engine or a complete TR-909 renderer.
- Branch review found one real bug before merge: zero drum-bus level was still forcing audible output through a fallback gain floor. That was fixed on-branch and covered by a new test before the PR was considered clean.
- Merge used local green verification plus a mergeable PR because the GitHub connector again reported an empty external status set in this environment.
