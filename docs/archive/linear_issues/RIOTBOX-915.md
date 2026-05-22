# `RIOTBOX-915` Extract Jam scene commit cue helpers from landed warnings shard

- Ticket: `RIOTBOX-915`
- Title: `Extract Jam scene commit cue helpers from landed warnings shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-915/extract-jam-scene-commit-cue-helpers-from-landed-warnings-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-915-scene-commit-cues-module`
- Linear branch: `feature/riotbox-915-extract-jam-scene-commit-cue-helpers-from-landed-warnings`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#908 (https://github.com/marang/riotbox/pull/908)`
- Merge commit: `605feb79304c7a3d0562b3658b6f748a7f297508`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app scene_post_commit; cargo test -p riotbox-app latest_landed_line_styles_define_result_hierarchy; cargo test -p riotbox-app next_panel_promotes_timing_rail_above_landed_history; cargo test -p riotbox-app renders_scene_restore_post_commit_guidance; cargo test -p riotbox-app renders_jam_shell_with_pending_scene; git diff --check; scripts/run_compact.sh /tmp/riotbox-915-ci.log just ci; GitHub Rust CI #2267 passed`
- Docs touched: `None`
- Follow-ups: `Continue with the next bounded TUI include/module cleanup from clean main.`

## Why This Ticket Existed

Reduce the largest remaining TUI shard by extracting its coherent Scene commit/history cue responsibility without changing musician-facing text.

## What Shipped

- ui::jam_scene_commit_cues now owns Scene history trail, latest landed command lookup, and post-commit Scene cue construction; jam_landed_warnings_source.rs dropped from 493 to 385 lines while Jam/Log scene text stayed unchanged.

## Notes

- Behavior-preserving semantic split only. No ActionCommand, JamAppState, Session/replay, lane, runtime, Scene behavior, visual redesign, or audio-output behavior changed.
