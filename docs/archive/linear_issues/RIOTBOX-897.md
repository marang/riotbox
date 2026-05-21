# `RIOTBOX-897` Use shared Source Timing primary warning in Jam Trust

- Ticket: `RIOTBOX-897`
- Title: `Use shared Source Timing primary warning in Jam Trust`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-897/use-shared-source-timing-primary-warning-in-jam-trust`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-897-shared-primary-warning-jam-trust`
- Linear branch: `feature/riotbox-897-use-shared-source-timing-primary-warning-in-jam-trust`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`, `ux`
- PR: `#890 (https://github.com/marang/riotbox/pull/890)`
- Merge commit: `6f6c1ae80b56a2daa437821051949a9782e63cd5`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests::jam_trust_warning_uses_shared_source_timing_priority; cargo test -p riotbox-app ui::tests; git diff --check origin/main...HEAD; just ci; GitHub Rust CI #2211 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Align Jam Trust warning language with the shared Source Timing primary-warning priority found in RIOTBOX-896.

## What Shipped

- Jam Trust now renders jam_view.source.timing.primary_warning and removed the local timing-warning label mapper; UI coverage proves sparse_onsets wins over lower-priority raw warning order.

## Notes

- None
