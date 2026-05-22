# `RIOTBOX-908` Convert audio runtime include shell into semantic Rust modules

- Ticket: `RIOTBOX-908`
- Title: `Convert audio runtime include shell into semantic Rust modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-908/convert-audio-runtime-include-shell-into-semantic-rust-modules`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-908-audio-runtime-semantic-modules`
- Linear branch: `feature/riotbox-908-convert-audio-runtime-include-shell-into-semantic-rust`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `review-followup`
- PR: `#901 (https://github.com/marang/riotbox/pull/901)`
- Merge commit: `88879030a25ff1a5e2bb7f9179be35e5ff2afa09`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-audio; git diff --check; cargo test -p riotbox-audio runtime; cargo test -p riotbox-audio; scripts/run_compact.sh /tmp/riotbox-908-audio-qa-ci.log just audio-qa-ci; scripts/run_compact.sh /tmp/riotbox-908-ci.log just ci; GitHub Rust CI #2246 passed`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-909 audits the TUI include shell later; explicit runtime submodule import cleanup can happen as touched-module cleanup`

## Why This Ticket Existed

Replace the production audio runtime textual include shell with durable semantic Rust module ownership.

## What Shipped

- Audio runtime production shards are now real Rust modules from runtime.rs, with public API re-exported and sibling runtime internals shared through pub(super) boundaries.

## Notes

- Behavior-preserving ownership conversion only; no DSP, timing policy, public API, ActionCommand, Session/replay, lane, or audio-output behavior changed.
