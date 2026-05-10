# `RIOTBOX-228` Explain contrast Scene launch target in queue and Log

- Ticket: `RIOTBOX-228`
- Title: `Explain contrast Scene launch target in queue and Log`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-228/explain-contrast-scene-launch-target-in-queue-and-log`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-228-scene-contrast-explanation`
- Linear branch: `feature/riotbox-228-explain-contrast-scene-launch-target-in-queue-and-log`
- PR: `#218`
- Merge commit: `24c929c`
- Labels: `ux`
- Follow-ups: `RIOTBOX-229`

## Why This Ticket Existed

`RIOTBOX-226` made Scene launch prefer an energy-contrast target when known Scene energy allows it. The selected target was visible, but queued and committed action text still looked like a generic ordered Scene jump, which made the contrast policy hard to verify from Jam and Log.

## What Shipped

- Added an explicit Scene launch target reason to the Jam view-model helper.
- Used contrast-specific queue wording when the policy skipped the immediate ordered target for a known energy contrast.
- Surfaced contrast wording in the committed action result without changing the action identity or generic fallback path.
- Preserved ordered fallback wording for missing, unknown, or non-contrast energy cases.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core prefers_contrast_next_scene_when_energy_data_is_available`
- `cargo test -p riotbox-app queue_scene_select_prefers_energy_contrast_candidate`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Explanation-only Scene Brain slice; no new target policy, audio behavior, persistence model, or broad Log redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
