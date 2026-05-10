# `RIOTBOX-214` Update TUI spec for Scene launch suggestion direction

- Ticket: `RIOTBOX-214`
- Title: `Update TUI spec for Scene launch suggestion direction`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-214/update-tui-spec-for-scene-launch-suggestion-direction`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-214-tui-spec-launch-suggestion-direction`
- Linear branch: `feature/riotbox-214-update-tui-spec-for-scene-launch-suggestion-direction`
- PR: `#204`
- Merge commit: `a62ffc8`
- Labels: `ux`
- Follow-ups: `RIOTBOX-215`

## Why This Ticket Existed

`RIOTBOX-213` made next Scene launch suggestions more explicit when a deterministic target and energy direction are available. The TUI spec needed to record that suggested gestures may include a compact target and `rise/drop/hold` direction while preserving fallback wording.

## What Shipped

- Documented suggested Scene launch gestures as part of the Jam screen performance contract.
- Specified that launch suggestions may name the deterministic next target and `rise/drop/hold` direction.
- Preserved the generic `[y] jump` fallback when the next target or direction is unknown.

## Verification

- `git diff --check`
- `rg -n "suggested Scene launch|generic \`[y] jump\` fallback|rise.*drop.*hold" docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/spec-only slice; no runtime behavior, UI implementation, screenshots, or broad TUI spec rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

## 2026-04-30 Linear Done/Duplicate Cleanup

Grouped archive entries for Linear issues that were already merged/done or closed as duplicates and removed from active Linear during the 2026-04-30 cleanup pass.
