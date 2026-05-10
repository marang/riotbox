# `RIOTBOX-212` Update TUI spec for Scene restore direction cue

- Ticket: `RIOTBOX-212`
- Title: `Update TUI spec for Scene restore direction cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-212/update-tui-spec-for-scene-restore-direction-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-212-tui-spec-restore-direction`
- Linear branch: `feature/riotbox-212-update-tui-spec-for-scene-restore-direction-cue`
- PR: `#202`
- Merge commit: `17f8922`
- Labels: `ux`
- Follow-ups: `RIOTBOX-213`

## Why This Ticket Existed

Recent Scene Brain slices made restore direction explicit in Jam cues and recipes. The TUI spec needed to record that `rise/drop/hold` cue as part of the intended Scene restore readability contract so future UI work does not regress it.

## What Shipped

- Documented restore-ready Scene cues as part of the Jam screen contract.
- Specified that restore affordances should show `rise/drop/hold` when both current and restore energies are known.
- Preserved the target-only fallback as intended behavior when energy data is incomplete.

## Verification

- `git diff --check`
- `rg -n "restore-ready|rise/drop/hold|target-only fallback" docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/spec-only slice; no runtime behavior, UI implementation, screenshots, or broad TUI spec rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
