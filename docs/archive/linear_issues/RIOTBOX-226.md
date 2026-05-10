# `RIOTBOX-226` Prefer contrast Scene launch target when energy data is available

- Ticket: `RIOTBOX-226`
- Title: `Prefer contrast Scene launch target when energy data is available`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-226/prefer-contrast-scene-launch-target-when-energy-data-is-available`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-226-scene-launch-contrast-target`
- Linear branch: `feature/riotbox-226-prefer-contrast-scene-launch-target-when-energy-data-is`
- PR: `#216`
- Merge commit: `0b59d3d`
- Labels: `ux`
- Follow-ups: `RIOTBOX-227`

## Why This Ticket Existed

Scene Brain moved through the ordered scene list. That was deterministic and replay-safe, but it did not yet use the existing section energy model to make launches feel more musical. The next visible Scene Brain step was to prefer an energy-contrast target when source analysis made one available.

## What Shipped

- Added a shared `next_scene_launch_candidate` policy in the core Jam view model path.
- Preferred the first deterministic different-energy Scene candidate when current and candidate energies are known.
- Preserved ordered fallback when energy is missing, `unknown`, or no contrast candidate exists.
- Routed app-level `queue_scene_select` through the same target policy as the Jam view.
- Documented the bounded contrast target rule in the TUI spec and recipes.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core view::jam::tests::`
- `cargo test -p riotbox-app queue_scene_select_`
- `git diff --check`
- `rg -n "contrast candidate|same-energy|next contrast scene|skip over" docs/specs/tui_screen_spec.md docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Bounded Scene target-policy slice only; no advanced arranger, Ghost policy, audio transition rendering, or source analysis model changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
