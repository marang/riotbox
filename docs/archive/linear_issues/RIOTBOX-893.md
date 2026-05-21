# `RIOTBOX-893` Require Recipe 15 real-source fixtures in the P012 all-lane proof gate

- Ticket: `RIOTBOX-893`
- Title: `Require Recipe 15 real-source fixtures in the P012 all-lane proof gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-893/require-recipe-15-real-source-fixtures-in-the-p012-all-lane-proof-gate`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Canceled`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-893-recipe15-strict-p012-gate`
- Linear branch: `feature/riotbox-893-require-recipe-15-real-source-fixtures-in-the-p012-all-lane`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`, `timing`
- PR: None
- Merge commit: `None`
- Deleted from Linear: `2026-05-21`
- Verification: `Verified current main already required Recipe 15 fixtures in the P012 gate; just recipe15-strict-missing-fixture-fixture passed during triage.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Review follow-up was opened for strict Recipe 15 fixture enforcement in the P012 all-lane proof gate.

## What Shipped

- Canceled as already satisfied on current main: p012-all-lane-source-grid-output-proof already invokes recipe15-feral-grid-auto-proof-strict with RIOTBOX_REQUIRE_RECIPE15_FIXTURES=1, and the strict missing-fixture fixture passes.

## Notes

- No PR was opened because the requested behavior was already implemented by prior commit 4656cb67.
