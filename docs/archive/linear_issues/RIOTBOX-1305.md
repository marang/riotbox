# `RIOTBOX-1305` P023: Strengthen W-30 hook/chop policy for first-two-bar impact

- Ticket: `RIOTBOX-1305`
- Title: `P023: Strengthen W-30 hook/chop policy for first-two-bar impact`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1305/p023-strengthen-w-30-hookchop-policy-for-first-two-bar-impact`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1305-p023-strengthen-w-30-hookchop-policy-for-first-two-bar-impact`
- Linear branch: `feature/riotbox-1305-p023-strengthen-w-30-hookchop-policy-for-first-two-bar`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1279 (https://github.com/marang/riotbox/pull/1279)`
- Merge commit: `5b9090b8`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py; git diff --check; just pro-pressure-source-matrix-smoke; just professional-source-wav-pack-smoke; just professional-output-listening-pack-smoke; just professional-output-suite-smoke; just ci; GitHub rust-ci passed`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 weak-output routing still ranked W-30 chop policy as a top production fix; tonal/dense first-two-bar material needed stronger source-transformed hook character without masking source or claiming quality proof.

## What Shipped

- Added source-character contrast selection for W-30 riff starts, recorded per-candidate source metrics, brought tonal-hook W-30 playback forward, rebalanced tonal rebuild pressure, kept dense/sparse pressure gates green, and documented the P023 boundary.

## Notes

- Reports remain diagnostic/listening-scaffold evidence with quality_proof false and human_verdict unverified; no hardcoded musical fallback was added.
