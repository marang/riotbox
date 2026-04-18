# `RIOTBOX-120` Clarify landed scene-result and next-step guidance after jump or restore

- Ticket: `RIOTBOX-120`
- Title: `Clarify landed scene-result and next-step guidance after jump or restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-120/clarify-landed-scene-result-and-next-step-guidance-after-jump-or`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-120-scene-next-step-guidance`
- Linear branch: `feature/riotbox-120-clarify-landed-scene-result-and-next-step-guidance-after`
- Assignee: `Markus`
- Labels: `None`
- PR: `#113`
- Merge commit: `f729c4bc8d63b79e5bd60b4c4a2f159feb91d481`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#319`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-121`, `RIOTBOX-122`, `RIOTBOX-123`

## Why This Ticket Existed

The Jam shell had started to explain queued scene timing, but it still fell back to generic post-commit guidance after a scene move landed. Scene Brain needed one bounded follow-up that explains what changed and what the next sensible action is after a jump or restore without forcing the player into `Log`.

## What Shipped

- detected recently landed `scene.launch` and `scene.restore` actions in the Jam shell
- added compact scene-specific `changed / next / then` guidance after those actions land
- kept the generic non-scene post-commit guidance unchanged
- added focused shell regressions for both landed jump and landed restore paths

## Notes

- the guidance deliberately uses compact scene labels (`drop`, `break`) so it fits the narrow perform column without wrapping
- this slice improved legibility, not scene semantics; it does not change queueing, restore behavior, or action history
