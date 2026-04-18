# `RIOTBOX-40` Ticket Archive

- Ticket: `RIOTBOX-40`
- Title: `Prepare audio-facing TR-909 reinforcement render seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-40/prepare-audio-facing-tr-909-reinforcement-render-seam`
- Project: `P005 | TR-909 MVP`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-40-tr909-render-seam`
- Linear branch: `feature/riotbox-40-prepare-audio-facing-tr-909-reinforcement-render-seam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#34`
- Merge commit: `699d06d`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`, `docs/README.md`, `docs/screenshots/jam_tr909_render_seam_baseline.txt`
- Follow-ups: `None yet; the next honest slice is audible TR-909 reinforcement behind the new render seam`

## Why This Ticket Existed

`RIOTBOX-39` left Riotbox with explicit TR-909 takeover and release actions, but there was still no audio-facing contract bridging committed TR-909 lane state into later audible reinforcement work. The roadmap needed one bounded seam before any honest drum rendering slice could start.

## What Shipped

- added a dedicated `riotbox-audio::tr909` render-state contract
- derived that render state in `riotbox-app` from committed TR-909 lane state plus transport and mixer context
- exposed the current render mode and routing in the Jam shell so the seam is reviewable in the UI
- added regression tests for idle, reinforce, takeover, and release render projections
- captured the review artifact at `docs/screenshots/jam_tr909_render_seam_baseline.txt`

## Notes

- The slice stayed deliberately short of actual drum synthesis.
- Merge used local green verification plus a mergeable PR because the GitHub connector reported an empty external status set in this environment.
- The important architectural constraint is preserved: later audible TR-909 work should consume this render seam rather than re-deriving state from shell-only cues or creating a parallel device-control system.
