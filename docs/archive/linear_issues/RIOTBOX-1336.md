# `RIOTBOX-1336` P023: Add Scene Brain audible lane-target and restore-contrast proof

- Ticket: `RIOTBOX-1336`
- Title: `P023: Add Scene Brain audible lane-target and restore-contrast proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1336/p023-add-scene-brain-audible-lane-target-and-restore-contrast-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-08-22`
- Finished: `2026-08-22`
- Branch: `feature/riotbox-1336-p023-add-scene-brain-audible-lane-target-and-restore`
- Linear branch: `feature/riotbox-1336-p023-add-scene-brain-audible-lane-target-and-restore`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1434 (https://github.com/marang/riotbox/pull/1434)`
- Merge commit: `342ede286414c14eed6d084c2ad1ec384beeab08`
- Deleted from Linear: `2026-08-22`
- Verification: `cargo test -p riotbox-core scene; cargo test -p riotbox-app scene; just p014-scene-movement-observer-probe; just audio-qa-ci; just ci; GitHub Rust CI: pass`
- Docs touched: `docs/specs/arrangement_scene_system_spec.md`, `docs/reviews/riotbox_1336_scene_w30_recall_development_stop_2026-08-22.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, `docs/research_decision_log.md`
- Follow-ups: `Any future Scene-owned W-30 recall or resample role requires a new Linear-first typed contract, Decision, bounded exploration, and product qualification.`

## Why This Ticket Existed

Scene movement had durable TR-909 and MC-202 intent but no explicit W-30 ownership, leaving room for implicit material replacement during launch and restore.

## What Shipped

- Added typed W-30 pin ownership through Session, legacy loading, replay, Jam view, observer evidence, pending and landed UI cues.
- Extended the P014 exact mixed-output proof to include W-30 and prove focused material remains sample-exact across launch, restore, and replay.
- Stopped and documented the rejected Development-only material-recall v1 without result-driven tuning; A remained usable but not preferred, while B was locally pitch- and groove-incoherent.

## Notes

- RBX-314 freezes Pin as explicit current ownership, not a new audible effect or broad P023 quality claim.
