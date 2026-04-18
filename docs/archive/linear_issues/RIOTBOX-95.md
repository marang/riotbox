# `RIOTBOX-95` Design a first-run onramp for the first meaningful play moment

- Ticket: `RIOTBOX-95`
- Title: `Design a first-run onramp for the first meaningful play moment`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-95/design-a-first-run-onramp-for-the-first-meaningful-play-moment`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-95-first-run-onramp`
- Linear branch: `feature/riotbox-95-design-a-first-run-onramp-for-the-first-meaningful-play`
- Assignee: `Markus`
- Labels: `None`
- PR: `#88`
- Merge commit: `688de53b79125acc052748caaf0abfe49b8d2406`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#245`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-94`

## Why This Ticket Existed

Real first-run feedback showed that Riotbox's current shell was already capable, but too open and too equal-rank for first contact. New users did not know what to do first, what Riotbox was doing with the source material, or how to get one obvious early success moment without reading through the full operator surface.

## What Shipped

- added a bounded `Start Here` onramp directly inside the existing Jam shell
- made the guidance evolve across three early states: start transport, first move queued, first result landed
- extended the help overlay with matching first-run hints
- added focused UI regressions for start, queued, committed, and help-overlay first-run states

## Notes

- the onramp is now gated by explicit fresh-ingest shell context rather than being inferred from arbitrary loaded session history
- this slice intentionally stayed inside the current Jam shell instead of creating a separate onboarding mode or wizard
- broader long-term Jam simplification still belongs to `RIOTBOX-94`
