# `RIOTBOX-295` Expose the source-backed capture-to-pad first play path in docs and shell cues

- Ticket: `RIOTBOX-295`
- Title: `Expose the source-backed capture-to-pad first play path in docs and shell cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-295/expose-the-source-backed-capture-to-pad-first-play-path-in-docs-and`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-295-expose-the-source-backed-capture-to-pad-first-play-path-in-docs-and-shell-cues`
- Linear branch: `feature/riotbox-295-expose-the-source-backed-capture-to-pad-first-play-path-in`
- PR: `#285`
- Merge commit: `a95adb8`
- Labels: `ux`
- Follow-ups: `RIOTBOX-296`

## Why This Ticket Existed

`RIOTBOX-294` locked down the app/runtime seam for source-backed capture-to-pad audition. The recipe layer still needed to teach that users must let capture and promotion commit before expecting `o` or `w` to test an audible W-30 result.

## What Shipped

- Retitled and tightened Recipe 3 around the capture -> raw audition -> promote -> W-30 hit path.
- Added raw audition and explicit wait-for-commit steps to Recipe 7.
- Updated Recipe 11 and README next-move guidance so users do not press through queued capture/promotion before audible results can exist.

## Verification

- `git diff --check`
- `rg -n "Capture, Audition|raw audition|capture -> raw audition|before the current action commits|audition -> promote" docs/jam_recipes.md README.md`
- `just ci`
- Branch diff reviewed with the `code-review` skill
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no runtime behavior, TUI controls, sequencing, audio rendering, or generated artifact convention changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
