# `RIOTBOX-118` Surface explicit scene action landing boundary on the Jam shell

- Ticket: `RIOTBOX-118`
- Title: `Surface explicit scene action landing boundary on the Jam shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-118/surface-explicit-scene-action-landing-boundary-on-the-jam-shell`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-118-scene-landing-boundary`
- Linear branch: `feature/riotbox-118-surface-explicit-scene-action-landing-boundary-on-the-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#111`
- Merge commit: `5fee5eb6175080f06e94b1ea17e5e0d6da265a25`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#315`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-119`, `RIOTBOX-120`, `RIOTBOX-121`

## Why This Ticket Existed

The first Scene Brain `jump` and `restore` path was already real, but the player still had to infer when those actions would land by reading the log or by waiting for the result. Riotbox needed one bounded Jam-shell cue that made the queued scene boundary explicit without opening a new diagnostics mode.

## What Shipped

- added a perform-facing quantization label helper for scene actions on the Jam shell
- changed the compact `Next` summary so queued scene actions now read like `launch -> scene-01-drop @ next bar`
- updated scene-brain shell regressions so the shorter boundary wording stays locked

## Notes

- this slice stayed intentionally Jam-only; it did not add a new timing widget or a separate inspect panel
- the lower `Pending / landed` surface still keeps its denser truth view, while the top `Next` panel now carries the player-facing boundary cue
