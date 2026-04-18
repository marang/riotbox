# `RIOTBOX-141` Add one compact post-landing scene cue after queued jump or restore commits

- Ticket: `RIOTBOX-141`
- Title: `Add one compact post-landing scene cue after queued jump or restore commits`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-141/add-one-compact-post-landing-scene-cue-after-queued-jump-or-restore`
- Project: `P008 | Scene Brain`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-141-post-landing-scene-cue`
- Linear branch: `feature/riotbox-141-add-one-compact-post-landing-scene-cue-after-queued-jump-or`
- Assignee: `Markus`
- Labels: `None`
- PR: `#135`
- Merge commit: `eeb2adaeafa161b002e07346a7ee704e8ca1bcee`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `PKG_CONFIG_PATH=/usr/lib/pkgconfig cargo test`, `PKG_CONFIG_PATH=/usr/lib/pkgconfig cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`, GitHub Actions `Rust CI` run `#365`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-143`, `RIOTBOX-144`, `RIOTBOX-145`

## Why This Ticket Existed

The Scene Brain readability stack had already taught queued `jump` and `restore` timing, but the default Jam shell still dropped back to a generic status line immediately after the scene move landed. Riotbox needed one tiny post-landing cue that told the performer what changed and what likely next move remained available without reopening the older, denser guidance stack.

## What Shipped

- replaced the generic Jam `status ...` line with a compact scene-specific post-commit cue when the latest landed action is `scene.launch` or `scene.restore`
- compressed the previous multi-line scene guidance into a single `scene ... | restore ... | next ...` cue
- kept the default suggested-gesture stack generic instead of branching into a larger scene-only block
- updated the Jam shell tests to assert the bounded post-landing cue behavior

## Notes

- this slice intentionally changed only the perform-facing surface, not the scene timing or queue semantics themselves
- the local verification needed `PKG_CONFIG_PATH=/usr/lib/pkgconfig` in this environment so ALSA discovery would succeed during the Rust checks
