# `RIOTBOX-122` Add compact current-vs-restore scene contrast cue on Jam

- Ticket: `RIOTBOX-122`
- Title: `Add compact current-vs-restore scene contrast cue on Jam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-122/add-compact-current-vs-restore-scene-contrast-cue-on-jam`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-122-scene-restore-contrast`
- Linear branch: `feature/riotbox-122-add-compact-current-vs-restore-scene-contrast-cue-on-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#117`
- Merge commit: `2872a167e22e65848a9fe638cdf92d91343e35d8`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#326`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-123`, `RIOTBOX-129`

## Why This Ticket Existed

After Scene Brain jump and restore cues became readable in time, the next confusion risk on `Jam` was understanding the two scene roles quickly. Riotbox needed one bounded contrast cue that shows the live scene and restore target at a glance without reopening a denser inspect surface.

## What Shipped

- added a compact `live <scene> <> restore <scene>` contrast line to the `Jam` `Now` panel
- kept the cue inside the existing perform-first shell instead of adding a separate detail block
- refreshed scene-focused shell expectations so the new contrast stays covered by the existing UI regression path

## Notes

- this was a bounded Scene Brain UX slice only; it did not change scene timing, restore semantics, or runtime behavior
- the cue intentionally stays compact so later timing and history work can layer on without turning `Jam` back into a dashboard
