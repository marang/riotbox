# `RIOTBOX-102` Surface recipe-guide and source-comparison learning paths more clearly from the shell docs

- Ticket: `RIOTBOX-102`
- Title: `Surface recipe-guide and source-comparison learning paths more clearly from the shell docs`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-102/surface-recipe-guide-and-source-comparison-learning-paths-more-clearly`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-102-learning-paths`
- Linear branch: `feature/riotbox-102-surface-recipe-guide-and-source-comparison-learning-paths`
- Assignee: `Markus`
- Labels: `None`
- PR: `#94`
- Merge commit: `de4d8ac7bf0046a31d83ee8a3bc7dca4a0553813`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#267`
- Docs touched: `README.md`, `docs/README.md`, `docs/jam_recipes.md`
- Follow-ups: `RIOTBOX-101`

## Why This Ticket Existed

Live feedback showed that the minimal first-run loop was useful but too narrow on its own. The repo already had a stronger recipe guide, but the next-step learning path was still too easy to miss after the first tiny success. Riotbox needed a bounded pass that made the existing learning routes more explicit without inventing a second onboarding system.

## What Shipped

- pointed the root `README.md` directly at `Recipe 2` and `Recipe 5` once the first loop succeeds
- added a `User Learning Path` section to `docs/README.md`
- surfaced the same next-step hint inside the Jam help overlay
- clarified in `docs/jam_recipes.md` that `Recipe 7` is a workflow-learning recipe, not a promise of immediate wide sound variety

## Notes

- this slice deliberately stayed documentation/help-only and did not change any action, lane, or audio behavior
- the Jam help overlay needed a slightly taller popup so the new next-step hint stayed visible in the existing shell snapshot size
- the motivating user report was valid: `Beat03` plus `Recipe 7` still collapses onto a narrow current preview path, so the honest fix here was to clarify the learning path rather than imply the user was pressing the wrong keys

