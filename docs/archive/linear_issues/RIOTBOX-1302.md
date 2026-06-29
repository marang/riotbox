# `RIOTBOX-1302` P023: Derive MC-202 pressure contour from source low-band movement

- Ticket: `RIOTBOX-1302`
- Title: `P023: Derive MC-202 pressure contour from source low-band movement`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1302/p023-derive-mc-202-pressure-contour-from-source-low-band-movement`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1302-p023-derive-mc-202-pressure-contour-from-source-low-band`
- Linear branch: `feature/riotbox-1302-p023-derive-mc-202-pressure-contour-from-source-low-band`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1276 (https://github.com/marang/riotbox/pull/1276)`
- Merge commit: `64bed56c5d9389de876ac26c3785e6c4b8ffb9c4`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt`; `cargo test -p riotbox-app committed_mc202_pressure_contour_tracks_source_low_band_movement -- --nocapture`; `cargo test -p riotbox-app mc202 -- --nocapture`; `cargo test -p riotbox-audio mc202 -- --nocapture`; `git diff --check main...HEAD`; `scripts/run_compact.sh /tmp/riotbox-1302-just-ci.log just ci`; `GitHub rust-ci passed on PR #1276`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-1301 tightened the MC-202 bass render proof, but SubPressureShove phrase content still needed source low-band movement to shape the actual pressure contour instead of staying close to a fixed two-hit gesture.

## What Shipped

- SubPressureShove now derives root, secondary, and optional pressure-movement notes from persisted source-expression low-band pressure / movement evidence.
- The groove map records a pressure-movement step in provenance so the pressure-contour decision is reviewable and replay-visible.
- A high-vs-low low-band movement regression now uses the same source fingerprint basis and proves different phrase cells plus measurable rendered output delta.

## Notes

- Human/demo promotion remains blocked; this is a source-composed implementation proof, not a human musical-pass claim.
