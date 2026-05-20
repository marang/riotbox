# `RIOTBOX-815` Add a bounded melodic source-chop showcase path for non-drum sources

- Ticket: `RIOTBOX-815`
- Title: `Add a bounded melodic source-chop showcase path for non-drum sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-815/add-a-bounded-melodic-source-chop-showcase-path-for-non-drum-sources`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-815-melodic-source-chop-showcase`
- Linear branch: `feature/riotbox-815-add-a-bounded-melodic-source-chop-showcase-path-for-non-drum`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#810 (https://github.com/marang/riotbox/pull/810)`
- Merge commit: `aba08f9cc8fb5f609eb8acda003ab5e862cb2e66`
- Verification: `GitHub Actions Rust CI #1969 passed; local just ci passed; local just audio-qa-ci passed; just melodic-source-chop-showcase passed with RushArp Source Timing unavailable boundary and non-silent W-30 source-chop output proof.`
- Docs touched: `docs/benchmarks/melodic_source_chop_showcase_2026-05-20.md; docs/benchmarks/README.md; docs/jam_recipes.md; docs/benchmarks/source_timing_example_readiness_2026-05-07.md`
- Follow-ups: `None from this slice; future P013 work can build on the bounded showcase path for richer melodic lane behavior.`

## Why This Ticket Existed

Non-drum melodic sources such as DH_RushArp_120_A.wav needed an honest showcase path that does not pretend Feral grid drum timing trust applies.

## What Shipped

- Added just melodic-source-chop-showcase and scripts/generate_melodic_source_chop_showcase.sh to validate Source Timing unavailable status, render the existing before/after pack, validate the manifest, and document the review path.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.
