# `RIOTBOX-109` Add replay-safe scene restore committed-state and shell regression fixtures

- Ticket: `RIOTBOX-109`
- Title: `Add replay-safe scene restore committed-state and shell regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-109/add-replay-safe-scene-restore-committed-state-and-shell-regression`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-109-scene-restore-fixtures`
- Linear branch: `feature/riotbox-109-add-replay-safe-scene-restore-committed-state-and-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#102`
- Merge commit: `0897f0c7011596c43636e35a5242bd7d19ab8325`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `GitHub Actions Rust CI #290`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-110`, `RIOTBOX-111`, `RIOTBOX-112`

## Why This Ticket Existed

`scene.restore` was already real and visible, but it still lacked the shared regression net that candidate projection and scene launch already had. Riotbox needed the restore seam covered by the same replay-safe fixture corpus so later Scene Brain work would not drift across app state and shell output.

## What Shipped

- extended the shared Scene Brain fixture corpus with a committed `scene.restore` case
- taught both app and shell fixture harnesses how to seed initial active/current/restore scene state before replaying a scene action
- asserted restore-pointer flips in committed state alongside shell-visible output from the same corpus
- updated the existing scene-launch fixture to preserve the restore-pointer expectation it already implied

## Notes

- this slice was verification-only and added no new scene behavior
- the fixture harness intentionally stayed generic enough to cover both launch and restore without opening a second test-only scene model

