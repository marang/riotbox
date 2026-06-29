# `RIOTBOX-1304` P023: Strengthen sparse MC-202 bass-pressure low-end body

- Ticket: `RIOTBOX-1304`
- Title: `P023: Strengthen sparse MC-202 bass-pressure low-end body`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1304/p023-strengthen-sparse-mc-202-bass-pressure-low-end-body`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1304-p023-strengthen-sparse-mc-202-bass-pressure-low-end-body`
- Linear branch: `feature/riotbox-1304-p023-strengthen-sparse-mc-202-bass-pressure-low-end-body`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1278 (https://github.com/marang/riotbox/pull/1278)`
- Merge commit: `fba4c1a191bd64932f830028405f575ec27f57b1`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo test -p riotbox-audio mc202 -- --nocapture; git diff --check; just pro-pressure-source-matrix-smoke; just professional-output-listening-pack-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `Continue P023 via next weak-output production-fix candidate from readiness/routing reports.`

## Why This Ticket Existed

Sparse MC-202 bass-pressure could pass technical gates while still reading too much like a midrange phrase; P023 needed source-derived low-end body proof without fallback output.

## What Shipped

- Added source-derived MC-202 low-body emphasis and deeper pressure reinforcement for low-dominant drop contours, exposed low_body_emphasis in manifests, tightened sparse/dense professional pressure gates, added regression coverage, and documented RIOTBOX-1304 in the roadmap.

## Notes

- None
