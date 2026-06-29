# `RIOTBOX-1322` P023: Migrate Source Graph include shell to semantic Rust modules

- Ticket: `RIOTBOX-1322`
- Title: `P023: Migrate Source Graph include shell to semantic Rust modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1322/p023-migrate-source-graph-include-shell-to-semantic-rust-modules`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1322-p023-migrate-source-graph-include-shell-to-semantic-rust`
- Linear branch: `feature/riotbox-1322-p023-migrate-source-graph-include-shell-to-semantic-rust`
- Assignee: `Markus`
- Labels: None
- PR: `#1296 (https://github.com/marang/riotbox/pull/1296)`
- Merge commit: `a59bf007f2190faddad51498fb905f05e3d1916b`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; scripts/check_no_textual_includes.sh; git diff --check; cargo test -p riotbox-core; cargo clippy -p riotbox-core --all-targets --all-features -- -D warnings; GitHub rust-ci pass`
- Docs touched: `docs/engineering/textual_include_allowlist.txt; docs/engineering/textual_include_inventory_2026-06-29.md`
- Follow-ups: `RIOTBOX-1330 or a dedicated follow-up should migrate the nested source_graph/timing_probe_candidates.rs include shell; RIOTBOX-1323 continues with Session module migration.`

## Why This Ticket Existed

The Source Graph root was still a textual include shell. P023 timing/source-backed work needs visible semantic module ownership without changing Source Graph behavior or public API.

## What Shipped

- Replaced crates/riotbox-core/src/source_graph.rs with source_graph/mod.rs real modules and pub-use compatibility exports, adjusted test module paths/imports, and updated the textual include allowlist/inventory from 256 to 246 include sites.

## Notes

- Behavior-preserving Rust module migration only. Nested timing_probe_candidates textual includes remain allowlisted for a later focused slice.
