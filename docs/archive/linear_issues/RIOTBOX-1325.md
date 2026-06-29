# `RIOTBOX-1325` P023: Thin riotbox-app binary into library CLI modules

- Ticket: `RIOTBOX-1325`
- Title: `P023: Thin riotbox-app binary into library CLI modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1325/p023-thin-riotbox-app-binary-into-library-cli-modules`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1325-p023-thin-riotbox-app-binary-into-library-cli-modules`
- Linear branch: `feature/riotbox-1325-p023-thin-riotbox-app-binary-into-library-cli-modules`
- Assignee: `Markus`
- Labels: None
- PR: `#1299 (https://github.com/marang/riotbox/pull/1299)`
- Merge commit: `d73d406909e8b0dece2adba42b8cb96b1b5bfb52`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; scripts/check_no_textual_includes.sh; git diff --check; cargo check -p riotbox-app; cargo test -p riotbox-app; cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings; cargo run -p riotbox-app --bin riotbox-app -- --help compatibility probe`
- Docs touched: `docs/engineering/textual_include_allowlist.txt; docs/engineering/textual_include_inventory_2026-06-29.md`
- Follow-ups: `RIOTBOX-1337 tracks semantic CLI include shell split.`

## Why This Ticket Existed

P023 needed the riotbox-app binary reduced to a thin entrypoint so later CLI ownership work happens in library-owned modules instead of a durable bin include shell.

## What Shipped

- Moved the CLI include shell behind riotbox_app::cli::run(), kept the binary as a small delegating main, preserved CLI behavior, and updated textual include ownership docs.

## Notes

- This intentionally preserved the existing include count and CLI behavior; semantic module ownership is split to RIOTBOX-1337.
