# `RIOTBOX-111` Add first scene energy summary on the existing Jam and Log shell surfaces

- Ticket: `RIOTBOX-111`
- Title: `Add first scene energy summary on the existing Jam and Log shell surfaces`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-111/add-first-scene-energy-summary-on-the-existing-jam-and-log-shell`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-111-scene-energy-summary`
- Linear branch: `feature/riotbox-111-add-first-scene-energy-summary-on-the-existing-jam-and-log`
- Assignee: `Markus`
- Labels: `None`
- PR: `#104`
- Merge commit: `f472f11567cbc040fa4acd2b6ba75bd1c88b2ca2`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#295`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-112`, `RIOTBOX-113`

## Why This Ticket Existed

Scene Brain already had scene ids, launch cues, restore cues, and replay-safe fixtures, but the default shell still reduced scene changes to labels alone. Riotbox needed one bounded energy-facing cue so scene changes could start to read as musical contrast instead of only as navigation state.

## What Shipped

- derived one compact current-scene energy label from the existing Scene Brain state and source graph section energy classes
- surfaced that energy cue in the Jam `Now` summary alongside the active scene label
- surfaced the same compact scene-and-energy summary in the Log `Counts` block
- updated the shared scene-shell regression fixture expectations so the new summary stays replay-safe

## Notes

- this slice stayed presentation-only and did not change scene selection, launch, or restore behavior
- the energy cue currently relies on the bounded Scene Brain scene ordering already used by the shell instead of introducing a new mapping model
