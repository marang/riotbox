# `RIOTBOX-835` Split observer/audio Source Timing alignment tests by evidence family

- Ticket: `RIOTBOX-835`
- Title: `Split observer/audio Source Timing alignment tests by evidence family`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-835/split-observeraudio-source-timing-alignment-tests-by-evidence-family`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-835-source-timing-alignment-tests-split`
- Linear branch: `feature/riotbox-835-split-observeraudio-source-timing-alignment-tests-by`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#830 (https://github.com/marang/riotbox/pull/830)`
- Merge commit: `01f003d41d40d5ce9d15beea922fcfd99a57040f`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-app observer_audio_correlate -- --nocapture`; `just ci`; `git diff --check`; GitHub Actions Rust CI #2029
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-834 pushed the observer/audio Source Timing alignment test file over the repo's soft 500-line Rust review budget, increasing review and agent-context cost before the next timing expansion.

## What Shipped

- Split Source Timing alignment tests by evidence family into core alignment/downbeat offset, anchor/groove evidence, and grid-use/policy modules while keeping shared fixture builders in the parent module. The parent file dropped from 637 to 240 lines; no behavior or schema changes.

## Notes

- None
