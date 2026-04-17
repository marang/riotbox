# `RIOTBOX-93` Create repo-root README with logo, quickstart, and musician-facing product overview

- Ticket: `RIOTBOX-93`
- Title: `Create repo-root README with logo, quickstart, and musician-facing product overview`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-93/create-repo-root-readme-with-logo-quickstart-and-musician-facing`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-93-readme-logo`
- Linear branch: `feature/riotbox-93-create-repo-root-readme-with-logo-quickstart-and-musician`
- Assignee: `Markus`
- Labels: `None`
- PR: `#87`
- Merge commit: `6a9e5c872942525f22c40f1d0f3f1823cab39f37`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#242`
- Docs touched: `README.md`, `docs/assets/riotbox-logo.svg`
- Follow-ups: `RIOTBOX-94`, `RIOTBOX-95`

## Why This Ticket Existed

The repo root was still effectively blank even though Riotbox had already grown into a real playable prototype with working TR-909, MC-202, W-30, Scene Brain, capture, and log behavior. New users had no strong first-contact explanation of what Riotbox is, what it can already do, how to start, or why the terminal is part of the product instead of just an implementation detail.

## What Shipped

- replaced the placeholder root `README.md` with a musician-facing product overview
- added a repo-native Riotbox SVG logo for the repo front door
- documented a simple five-step quickstart using the included WAV examples
- tightened the copy around the first meaningful play moment, current screens, key actions, and prototype boundary

## Notes

- this slice intentionally improved the repo front door only; it did not add a first-run mode inside the TUI
- the later UX feedback about a reduced perform-first Jam surface and a clearer onboarding path is already tracked separately in `RIOTBOX-94` and `RIOTBOX-95`
