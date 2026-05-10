# `RIOTBOX-243` Cover Scene restore TR-909 source-support accent projection

- Ticket: `RIOTBOX-243`
- Title: `Cover Scene restore TR-909 source-support accent projection`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-243/cover-scene-restore-tr-909-source-support-accent-projection`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-243-scene-restore-tr909-accent`
- Linear branch: `feature/riotbox-243-cover-scene-restore-tr-909-source-support-accent-projection`
- PR: `#233`
- Merge commit: `0db1dfa`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-244`

## Why This Ticket Existed

Scene launch already had coverage proving that the target Scene projects into TR-909 source support as `scene_target` with `accent scene`. Scene restore uses the same performance recovery seam, so it needed its own focused regression to prevent future drift where launch stays coupled but restore silently falls back.

## What Shipped

- Added a focused `scene.restore` regression with TR-909 `SourceSupport` active.
- Asserted the restored Scene becomes the audio-facing current Scene.
- Asserted the restored Scene projects as `scene_target`.
- Asserted the runtime diagnostic accent becomes `scene`.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-app committed_scene_restore_projects_target_scene_into_tr909_source_support -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Regression-only slice; the existing runtime path already satisfied the contract, so no behavior change was needed.
- No transition engine, Scene policy, audio mix tuning, or TUI layout changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
