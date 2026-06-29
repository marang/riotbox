# `RIOTBOX-1310` P023: Tighten fixture thresholds for weak-output promotion boundaries

- Ticket: `RIOTBOX-1310`
- Title: `P023: Tighten fixture thresholds for weak-output promotion boundaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1310/p023-tighten-fixture-thresholds-for-weak-output-promotion-boundaries`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1310-p023-tighten-fixture-thresholds-for-weak-output-promotion-boundaries`
- Linear branch: `feature/riotbox-1310-p023-tighten-fixture-thresholds-for-weak-output-promotion`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1284 (https://github.com/marang/riotbox/pull/1284)`
- Merge commit: `851f872f272f33401728e11a1b893bfaaf3ac8b5`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt; cargo test -p riotbox-audio --bin feral_grid_pack tr909_rendered_drum_pressure -- --nocapture; cargo test -p riotbox-audio --bin feral_grid_pack source_aware_tr909_profile_changes_for_same_bpm_sources -- --nocapture; just syncopated-source-showcase-smoke; just professional-output-suite-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

TR-909 rendered drum-pressure evidence was overfitted to dense-break floors, causing tonal and syncopated fixtures to fail or pass for the wrong musical reason.

## What Shipped

- Profile-aware rendered drum-pressure thresholds now store per-case floors in manifests, validate professional-suite cases against those floors, keep tonal highs in steady_pulse unless real transient density exists, and split break_lift low-band expectations from stricter drop_drive pressure.

## Notes

- BreakLift keeps contribution pressure strict while using a snare/transient-appropriate low-band floor; DropDrive remains on the stricter low-pressure floor.
