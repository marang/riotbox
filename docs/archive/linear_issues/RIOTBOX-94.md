# `RIOTBOX-94` Reframe Jam screen into clearer perform-first vs inspect-first surfaces

- Ticket: `RIOTBOX-94`
- Title: `Reframe Jam screen into clearer perform-first vs inspect-first surfaces`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-94/reframe-jam-screen-into-clearer-perform-first-vs-inspect-first`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-94-perform-first-jam`
- Linear branch: `feature/riotbox-94-reframe-jam-screen-into-clearer-perform-first-vs-inspect`
- Assignee: `Markus`
- Labels: `None`
- PR: `#89`
- Merge commit: `ccd9bff362cb8b4b36467df97b7c4445f3ba02ba`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#251`
- Docs touched: `docs/research_decision_log.md`, `docs/screenshots/jam_perform_first_baseline.txt`, `docs/README.md`
- Follow-ups: `RIOTBOX-97`, `RIOTBOX-98`, `RIOTBOX-99`

## Why This Ticket Existed

The Jam shell had already become structurally rich, but the primary surface still felt too much like an operator dashboard. Too many equally loud diagnostics, lane details, and support cues were competing on the main performance screen, which made Riotbox harder to read as an instrument in the moment.

## What Shipped

- reduced the main Jam surface to a perform-first hierarchy built around `Now`, `Next`, and `Trust`
- replaced the old `Source / Sections / Macros` row with compact `MC-202`, `W-30`, and `TR-909` lane cards
- replaced the lower Jam region with `Pending / landed`, `Suggested gestures`, and `Warnings / trust`
- shortened footer and help wording into clearer primary-vs-secondary gesture groupings
- updated fixture-backed Jam shell regressions and added a normalized review baseline artifact for the new hierarchy

## Notes

- this slice intentionally kept a single Jam surface instead of adding a separate inspect mode yet
- the deeper analysis and lineage views remain on `Log`, `Source`, `Capture`, and `Help`
- the next UX questions are now split cleanly into inspect depth, first-30-seconds flow, and language polish rather than one overloaded Jam redesign ticket
