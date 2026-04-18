# `RIOTBOX-115` Centralize perform-facing gesture vocabulary across Jam shell surfaces

- Ticket: `RIOTBOX-115`
- Title: `Centralize perform-facing gesture vocabulary across Jam shell surfaces`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-115/centralize-perform-facing-gesture-vocabulary-across-jam-shell-surfaces`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-115-gesture-vocabulary`
- Linear branch: `feature/riotbox-115-centralize-perform-facing-gesture-vocabulary-across-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#108`
- Merge commit: `e7918efa66fb8954836513c348fd39ace7e82243`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#307`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Jam shell had already shifted toward more perform-facing language, but the wording still lived in several parallel string tables across status messages, footer rows, help content, and action summaries. Riotbox needed one bounded cleanup pass so that the learning path and the Scene Brain shell surfaces would stop drifting linguistically while the product vocabulary was still settling.

## What Shipped

- introduced one shared perform-facing gesture vocabulary table inside the shell layer
- generated footer, help, and queued-status copy from the same vocabulary source instead of repeating literal strings
- pointed action-summary labels at the same shared gesture names so landed and pending summaries match the visible controls
- kept the slice bounded to wording consistency only, without changing gesture semantics or adding new interactions

## Notes

- this was a shell-language alignment slice, not a broader Jam redesign
- deeper `Log` diagnostics remained technical on purpose while the perform-facing shell surfaces moved toward one shared vocabulary
