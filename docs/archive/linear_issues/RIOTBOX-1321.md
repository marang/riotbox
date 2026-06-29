# `RIOTBOX-1321` P023: Inventory Rust textual includes and add module-policy guardrail

- Ticket: `RIOTBOX-1321`
- Title: `P023: Inventory Rust textual includes and add module-policy guardrail`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1321/p023-inventory-rust-textual-includes-and-add-module-policy-guardrail`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1321-p023-inventory-rust-textual-includes-and-add-module-policy`
- Linear branch: `feature/riotbox-1321-p023-inventory-rust-textual-includes-and-add-module-policy`
- Assignee: `Markus`
- Labels: `Docs`
- PR: `#1295 (https://github.com/marang/riotbox/pull/1295)`
- Merge commit: `8e5c5266f0f32279b6544dd0356432e97472191b`
- Deleted from Linear: `2026-06-29`
- Verification: `scripts/check_no_textual_includes.sh; bash -n scripts/check_no_textual_includes.sh; git diff --check; GitHub rust-ci pass`
- Docs touched: `docs/README.md; docs/engineering/module_policy.md; docs/engineering/textual_include_inventory_2026-06-29.md; docs/engineering/textual_include_allowlist.txt`
- Follow-ups: `RIOTBOX-1322 Source Graph module migration; RIOTBOX-1323 Session module migration; RIOTBOX-1324 audio MC-202/source-audio module migration; RIOTBOX-1325 app binary CLI module split.`

## Why This Ticket Existed

Riotbox needed a concrete inventory and manual guardrail before migrating textual include shells. The project has many legacy include sites, and new ones should not appear unnoticed while behavior-preserving module migrations proceed.

## What Shipped

- Inventoried 256 textual include sites across 21 owning Rust files, added a counts-based allowlist, added scripts/check_no_textual_includes.sh, and linked the guardrail from module policy and docs README.

## Notes

- No behavior or module migration changed in this slice. The guardrail is manual and allowlist-based until migrations reduce the legacy surface enough for a harder CI gate.
