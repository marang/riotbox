# `RIOTBOX-872` Make Recipe 15 strict when invoked by the P012 all-lane gate

- Ticket: `RIOTBOX-872`
- Title: `Make Recipe 15 strict when invoked by the P012 all-lane gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-872/make-recipe-15-strict-when-invoked-by-the-p012-all-lane-gate`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-872-make-recipe-15-strict-when-invoked-by-p012-all-lane-gate`
- Linear branch: `feature/riotbox-872-make-recipe-15-strict-when-invoked-by-the-p012-all-lane-gate`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`, `timing`
- PR: `#866 (https://github.com/marang/riotbox/pull/866)`
- Merge commit: `5d72d7ae9d70be21985aeed8d263b3961fc76a8c`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check`; `just recipe15-strict-missing-fixture-fixture`; `just p012-all-lane-source-grid-output-proof`; `just ci`; GitHub Rust CI
- Docs touched: `docs/jam_recipes.md`, `docs/execution_roadmap.md`
- Follow-ups: `RIOTBOX-873`

## Why This Ticket Existed

Make the phase-level P012 all-lane gate honest after RIOTBOX-871 found that Recipe 15 real-source fixtures could be skipped with exit 0.

## What Shipped

- Added strict Recipe 15 fixture mode via RIOTBOX_REQUIRE_RECIPE15_FIXTURES=1.
- Kept standalone Recipe 15 proof skip-tolerant by default for optional local recipe exploration.
- Added recipe15-feral-grid-auto-proof-strict and recipe15-strict-missing-fixture-fixture.
- Made p012-all-lane-source-grid-output-proof call the strict Recipe 15 proof.
- Updated Recipe 15 and roadmap docs to state that missing required fixtures fail the phase-level gate.

## Notes

- None
