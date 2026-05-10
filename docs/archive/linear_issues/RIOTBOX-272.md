# `RIOTBOX-272` Add typed Capture target kind to Jam view

- Ticket: `RIOTBOX-272`
- Title: `Add typed Capture target kind to Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-272/add-typed-capture-target-kind-to-jam-view`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-272-add-typed-capture-target-kind-to-jam-view`
- Linear branch: `feature/riotbox-272-add-typed-capture-target-kind-to-jam-view`
- PR: `#262`
- Merge commit: `b4681df`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-273`

## Why This Ticket Existed

The W-30 Capture seam review found that Capture `Do Next` and the `hear ...` label branched on formatted target strings such as `pad ...` and `scene ...`. Display wording was carrying routing semantics.

## What Shipped

- Added typed `CaptureTargetKindView` to the Jam Capture summary alongside the existing display label.
- Updated Capture `Do Next` and `capture_heard_path_label` to branch on target kind instead of label prefixes.
- Added focused regression assertions for unassigned, W-30 pad, and Scene target projection.
- Recorded the routing decision in `docs/research_decision_log.md`.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-core builds_minimal_jam_view_model`
- `cargo test -p riotbox-app committed_promotion_actions_assign_target_to_existing_capture`
- `cargo test -p riotbox-app renders_capture_heard_path_for_scene_targets_without_w30_audition_keys`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- View-model/UI seam slice only; no persistence, audio behavior, keymap, or Capture layout redesign changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
