# `RIOTBOX-114` Promote scene energy mapping out of the TUI-only positional derivation

- Ticket: `RIOTBOX-114`
- Title: `Promote scene energy mapping out of the TUI-only positional derivation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-114/promote-scene-energy-mapping-out-of-the-tui-only-positional-derivation`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-114-scene-energy-contract`
- Linear branch: `feature/riotbox-114-promote-scene-energy-mapping-out-of-the-tui-only-positional`
- Assignee: `Markus`
- Labels: `None`
- PR: `#107`
- Merge commit: `7559ce599b8d233885b2712cd777ef0ccaa428fd`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#305`
- Docs touched: `docs/reviews/periodic_codebase_review_2026-04-18.md`
- Follow-ups: `RIOTBOX-115`

## Why This Ticket Existed

The first scene-energy cue was already useful on the shell, but it still lived as a TUI-only helper that guessed by matching the current scene’s position inside `scene_state.scenes`. Riotbox needed one bounded contract step that moved scene energy into the shared Jam view and stopped tying the cue to the shell’s own private derivation path.

## What Shipped

- added `active_scene_energy` to the shared `SceneSummaryView` in the core Jam view contract
- moved current-scene energy derivation into the core/app presentation layer instead of leaving it in a TUI helper
- switched the first lookup path to explicit projected scene-id numbering, with a bounded first-section fallback for older or generic scene ids
- updated app and shell tests so the shared scene-energy field stays covered across ingest, view-model, and shell rendering paths

## Notes

- this slice kept the current Scene Brain model bounded; it did not introduce a richer scene-to-section persistence model yet
- the explicit projected-scene-id path is now the primary contract, while the first-section fallback preserves current older test paths without restoring TUI-only positional matching
