# `RIOTBOX-89` Derive first bounded scene candidates from analyzed source sections

- Ticket: `RIOTBOX-89`
- Title: `Derive first bounded scene candidates from analyzed source sections`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-89/derive-first-bounded-scene-candidates-from-analyzed-source-sections`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-89-scene-candidates`
- Linear branch: `feature/riotbox-89-derive-first-bounded-scene-candidates-from-analyzed-source`
- Assignee: `Markus`
- Labels: `None`
- PR: `#83`
- Merge commit: `dd66b219545b5d210210046f1ce88fa88e743768`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#230`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-90`, `RIOTBOX-91`, `RIOTBOX-92`

## Why This Ticket Existed

The W-30 MVP had reached a natural phase boundary: Riotbox already had explicit session scene state, a transport scene pointer, and Jam-shell scene visibility, but those fields were still mostly placeholders. The first honest `Scene Brain` step was to make scene candidates real from analyzed source structure without opening a second arrangement or scene-graph model.

## What Shipped

- derived deterministic scene candidate IDs from ordered `SourceGraph.sections`
- projected those candidates into the existing session `scene_state.scenes`
- normalized `active_scene` and transport `current_scene` onto that same committed state when no scene was already set
- covered both empty-session normalization and persisted ingest state with focused tests

## Notes

- this slice is intentionally bounded to candidate projection on the existing session seam; it does not add scene-select, launch, restore, or transition logic yet
- later `Scene Brain` work should deepen the same session and transport state instead of inventing a second arrangement inventory
