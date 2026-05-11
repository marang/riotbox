# `RIOTBOX-779` Add Source Timing grid-use to app observer snapshots

- Ticket: `RIOTBOX-779`
- Title: `Add Source Timing grid-use to app observer snapshots`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-779/add-source-timing-grid-use-to-app-observer-snapshots`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-779-add-source-timing-grid-use-to-app-observer-snapshots`
- Linear branch: `feature/riotbox-779-add-source-timing-grid-use-to-app-observer-snapshots`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#773 (https://github.com/marang/riotbox/pull/773)`
- Merge commit: `5d50802840790a34e9670c2079c823c922b8b31e`
- Deleted from Linear: `2026-05-11`
- Verification: `just ci passed locally; GitHub Rust CI #1855 passed`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-780, RIOTBOX-781, RIOTBOX-782`

## Why This Ticket Existed

App observer source_timing snapshots lagged behind the shared Jam/Source grid_use contract, leaving the control-path observer event one step behind TUI and output-path QA surfaces.

## What Shipped

- Added source_timing.grid_use to app observer snapshots, observer/audio markdown and JSON summaries, NDJSON and summary validators, focused invalid fixtures, and Source Timing spec coverage.

## Notes

- Contract/QA plumbing only; no analyzer thresholds, lane behavior, TUI layout, or audio rendering behavior changed.
