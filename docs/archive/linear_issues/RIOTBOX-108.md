# `RIOTBOX-108` Surface scene restore target and pending restore cues in Jam and Log

- Ticket: `RIOTBOX-108`
- Title: `Surface scene restore target and pending restore cues in Jam and Log`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-108/surface-scene-restore-target-and-pending-restore-cues-in-jam-and-log`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-108-scene-restore-cues`
- Linear branch: `feature/riotbox-108-surface-scene-restore-target-and-pending-restore-cues-in-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#101`
- Merge commit: `ee3bb0afe592e311357a691db6d2266051ab6b93`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `GitHub Actions Rust CI #285`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-109`, `RIOTBOX-110`

## Why This Ticket Existed

Once `scene.restore` existed, the shell still did not make the recovery target legible enough in the perform-facing surfaces. Riotbox needed one presentation-only pass so the operator could see both the current restore target and a queued restore without leaving the existing Jam and Log spine.

## What Shipped

- surfaced the committed restore target directly in the Jam `Now` panel
- shortened the Log counts block so scene and restore labels stay visible instead of wrapping the restore cue away
- added a dedicated Log regression for queued restore visibility
- updated the shared Scene Brain fixture expectations to match the more stable scene/restore wording

## Notes

- this slice stayed presentation-only and did not add any new scene action semantics
- restore truth still comes from the committed session state and the normal queue, not from a second shell-only scene model
