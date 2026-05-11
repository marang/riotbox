# `RIOTBOX-763` Show a musician-readable Source Timing rail in Jam

- Ticket: `RIOTBOX-763`
- Title: `Show a musician-readable Source Timing rail in Jam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-763/show-a-musician-readable-source-timing-rail-in-jam`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-763-show-source-timing-rail-in-jam`
- Linear branch: `feature/riotbox-763-show-a-musician-readable-source-timing-rail-in-jam`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#759 (https://github.com/marang/riotbox/pull/759)`
- Merge commit: `bc857a7100108df6d0d18ac2cfa6b8056ac23b8f`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --lib -- --nocapture; cargo clippy -p riotbox-app --lib -- -D warnings; git diff --check; GitHub Actions Rust CI passed on PR #759`
- Docs touched: `docs/jam_recipes.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Jam perform mode needed a musician-readable source timing rail so users can see timing trust and the next bar boundary without opening Inspect or Source.

## What Shipped

- Added a compact Jam Now timing rail using the existing shared Source Timing summary; preserved beat/bar/phrase in the existing source clock; kept Ghost blocker context visible after the extra row; updated Jam recipe and TUI spec docs.

## Notes

- None
