# `RIOTBOX-1301` P023: Strengthen MC-202 source-composed bass movement render proof

- Ticket: `RIOTBOX-1301`
- Title: `P023: Strengthen MC-202 source-composed bass movement render proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1301/p023-strengthen-mc-202-source-composed-bass-movement-render-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1301-p023-strengthen-mc-202-source-composed-bass-movement-render`
- Linear branch: `feature/riotbox-1301-p023-strengthen-mc-202-source-composed-bass-movement-render`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1275 (https://github.com/marang/riotbox/pull/1275)`
- Merge commit: `3edc3f02d5ccac43cb064b2db2cc0cdfd08c80b9`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo test -p riotbox-app mc202 -- --nocapture; cargo test -p riotbox-audio mc202 -- --nocapture; git diff --check; just ci; GitHub rust-ci passed on PR #1275`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `RIOTBOX-1302 continues MC-202 pressure phrase composition from source low-band movement under RIOTBOX-1264.`

## Why This Ticket Existed

Strengthen MC-202 source-composed bass movement proof so pressure plans cannot pass while rendering weak, midrange-only, static, or fallback-like output.

## What Shipped

- SubPressureShove render projection carries bass body from persisted source-expression low-band evidence with legacy scorecard fallback; pressure-vs-hook tests require bass-articulation and low-band RMS/share margin; neutralized source evidence remains silent/degraded instead of leaking MC-202 fallback output.

## Notes

- Human/demo promotion remains blocked; this is producer-grade render proof, not a human listening pass.
