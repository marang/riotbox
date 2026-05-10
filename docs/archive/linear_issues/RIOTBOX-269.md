# `RIOTBOX-269` Refresh Capture recipes for audible audition cues

- Ticket: `RIOTBOX-269`
- Title: `Refresh Capture recipes for audible audition cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-269/refresh-capture-recipes-for-audible-audition-cues`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-269-refresh-capture-recipes-for-audible-audition-cues`
- Linear branch: `feature/riotbox-269-refresh-capture-recipes-for-audible-audition-cues`
- PR: `#259`
- Merge commit: `08e3fd8`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-270`

## Why This Ticket Existed

`RIOTBOX-268` tightened the Capture screen wording around `hear it`, raw audition, promoted audition, and the next audible handoff. The recipe docs needed to teach the current wording so a user following the examples knows what to look for in `Capture`.

## What Shipped

- Updated the Capture/reuse recipe to mention optional raw audition from the new `Do Next` wording.
- Taught the `hear it` / `keep it` / `play it` path now shown by the Capture screen.
- Added the queued-audition `wait, then hear ... preview` cue to the source-backed W-30 recipe expectations.

## Verification

- `git diff --check`
- `rg -n "hear it|keep it|play it|wait, then hear raw preview|wait, then hear promoted preview|audible handoff" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only learning-path slice; no UI implementation, new recipes, or new audio behavior claims changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
