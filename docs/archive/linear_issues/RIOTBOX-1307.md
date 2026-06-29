# `RIOTBOX-1307` P023: Strengthen mix-bus source clarity against generated masking

- Ticket: `RIOTBOX-1307`
- Title: `P023: Strengthen mix-bus source clarity against generated masking`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1307/p023-strengthen-mix-bus-source-clarity-against-generated-masking`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1307-p023-strengthen-mix-bus-source-clarity-against-generated-masking`
- Linear branch: `feature/riotbox-1307-p023-strengthen-mix-bus-source-clarity-against-generated`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1281 (https://github.com/marang/riotbox/pull/1281)`
- Merge commit: `46c34323`
- Deleted from Linear: `2026-06-29`
- Verification: `just ci; GitHub rust-ci; professional output suite max source-first generated/source ratio 0.035148393 under 0.08 with no failure codes`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `Continue P023 weak-output routing slices: source selection, drum pressure, fixture threshold, and UI cue remain candidates.`

## Why This Ticket Existed

P023 weak-output routing flagged generated support masking source-first identity; Riotbox needs source character audible, not generic generated backing.

## What Shipped

- Reduced source-first generated support bleed, raised W-30/source weight, tightened source-first generated/source ceiling from 0.16 to 0.08 across product and QA validators, updated tonal fixture, and recorded the boundary in the roadmap.

## Notes

- Evidence remains diagnostic with human_verdict: unverified and quality_proof: false until structured listening accepts it.
