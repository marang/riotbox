# `RIOTBOX-249` Document Scene post-commit cue hierarchy in TUI spec

- Ticket: `RIOTBOX-249`
- Title: `Document Scene post-commit cue hierarchy in TUI spec`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-249/document-scene-post-commit-cue-hierarchy-in-tui-spec`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-249-document-scene-post-commit-cue-hierarchy-in-tui-spec`
- Linear branch: `feature/riotbox-249-document-scene-post-commit-cue-hierarchy-in-tui-spec`
- PR: `#239`
- Merge commit: `480be4e`
- Labels: `ux`
- Follow-ups: `RIOTBOX-250`

## Why This Ticket Existed

`RIOTBOX-248` styled the Scene post-commit Jam cue so the current Scene, restore target, compact `909 lift` hint, and next keys no longer read as one flat diagnostic line. The TUI spec needed to record that hierarchy as a contract, not only an implementation detail.

## What Shipped

- Documented post-commit Scene cues in the Jam screen requirements.
- Added a Scene post-commit note to the first terminal emphasis tokens.
- Preserved monochrome readability as the baseline contract.
- Kept `909 lift` scoped as a compact support hint, not a new control.

## Verification

- `git diff --check main...HEAD`
- `rg -n 'post-commit Scene cues|909 lift|Scene post-commit cues|instrument line' docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Spec-only slice; no runtime behavior, screenshot baselines, broad TUI rewrites, or new color semantics changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
