# `RIOTBOX-241` Update TUI spec for TR-909 accent diagnostics

- Ticket: `RIOTBOX-241`
- Title: `Update TUI spec for TR-909 accent diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-241/update-tui-spec-for-tr-909-accent-diagnostics`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-241-tui-spec-tr909-accent`
- Linear branch: `feature/riotbox-241-update-tui-spec-for-tr-909-accent-diagnostics`
- PR: `#231`
- Merge commit: `f31b686`
- Labels: `ux`, `benchmark`
- Follow-ups: `RIOTBOX-242`

## Why This Ticket Existed

`RIOTBOX-238` surfaced compact TR-909 support accent cues in Log/Inspect and `RIOTBOX-239` taught them in recipes. The TUI spec needed to record that contract so later Jam/Log simplification keeps the cue visible without promoting it into a new control path.

## What Shipped

- Documented compact TR-909 support context and accent cues for Log diagnostics.
- Defined `accent scene`, `accent off fallback`, and non-source-support `off` wording.
- Clarified that Jam Inspect may show the same profile/context/accent/route tuple for diagnosis.
- Kept the accent cue read-only and explicitly outside transition-engine promises.

## Verification

- `git diff --check`
- `rg -n "accent scene|accent off fallback|profile / context / accent / route|transition-engine promise|TR-909 render diagnostics" docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/spec-only slice; no runtime behavior, audio behavior, TUI layout, screenshot baseline, or recipe wording changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
