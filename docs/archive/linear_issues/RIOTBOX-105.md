# `RIOTBOX-105` Compress Jam lane cards to current and next summaries

- Ticket: `RIOTBOX-105`
- Title: `Compress Jam lane cards to current and next summaries`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-105/compress-jam-lane-cards-to-current-and-next-summaries`
- Project: `Riotbox MVP Buildout`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-105-jam-card-compression`
- Linear branch: `feature/riotbox-105-compress-jam-lane-cards-to-current-and-next-summaries`
- Assignee: `Markus`
- Labels: `None`
- PR: `#99`
- Merge commit: `86253b3c5ece344e07d9d491d2169097d41985d0`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#281`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Jam surface had already become more perform-first, but the individual lane cards were still carrying too much operator detail. Riotbox needed one bounded reduction pass that kept the lane metaphor intact while making each card answer two questions first: what is happening now, and what is the next queued change?

## What Shipped

- reduced the MC-202 Jam card to current voice, next change, and current phrase
- reduced the W-30 Jam card to current pad, current preview state, and next cue
- reduced the TR-909 Jam card to current mode, current fill/slam state, and next change
- removed perform-surface-only helper copy that no longer had a role on the compressed Jam surface
- updated fixture-backed shell expectations and focused Jam assertions to lock the new `current / next` layout in place

## Notes

- this slice changed presentation only; it did not change queue semantics, runtime behavior, or inspect/log/capture surfaces
- deeper W-30 manager and resample-tap diagnostics deliberately remained available outside the compressed perform cards
- this landed as the last bounded Jam-card density reduction pass before the next workflow-focused onboarding improvements
