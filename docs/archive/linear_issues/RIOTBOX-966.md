# `RIOTBOX-966` Add observer beat/bar counts to generated Feral-grid summary index

- Ticket: `RIOTBOX-966`
- Title: `Add observer beat/bar counts to generated Feral-grid summary index`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-966/add-observer-beatbar-counts-to-generated-feral-grid-summary-index`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-23`
- Branch: `feature/riotbox-966-add-observer-beatbar-counts-to-generated-feral-grid-summary`
- Linear branch: `feature/riotbox-966-add-observer-beatbar-counts-to-generated-feral-grid-summary`
- Assignee: `Markus`
- Labels: None
- PR: `#959 (https://github.com/marang/riotbox/pull/959)`
- Merge commit: `03fedb2a955ba0c07911e78e1389720454294a2e`
- Deleted from Linear: `2026-05-23`
- Verification: `bash -n scripts/correlate_generated_feral_grid_observer.sh; git diff --check; just observer-audio-correlate-generated-feral-grid; just ci; GitHub Actions Rust CI run 26307127157 success`
- Docs touched: `None`
- Follow-ups: `Source transport/map/capture planning begins next from clean main`

## Why This Ticket Existed

Expose observer beat/bar count evidence in the lower-level generated Feral-grid summary index.

## What Shipped

- Added observer_beat_count and observer_bar_count columns from control_path.observer_source_timing and updated generated summary assertions.

## Notes

- None
