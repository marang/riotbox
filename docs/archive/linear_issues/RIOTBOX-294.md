# `RIOTBOX-294` Add source-backed capture-to-pad preview flow regression

- Ticket: `RIOTBOX-294`
- Title: `Add source-backed capture-to-pad preview flow regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-294/add-source-backed-capture-to-pad-preview-flow-regression`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-294-add-source-backed-capture-to-pad-preview-flow-regression`
- Linear branch: `feature/riotbox-294-add-source-backed-capture-to-pad-preview-flow-regression`
- PR: `#284`
- Merge commit: `6071352`
- Labels: `benchmark`
- Follow-ups: `RIOTBOX-295`

## Why This Ticket Existed

The app could load source audio into the W-30 preview cache, and committed captures carried source-window provenance. The missing guard was the complete musician-facing seam: capture source material, promote it to a W-30 pad, and audition that pad with non-empty source-backed preview samples.

## What Shipped

- Added an app-level regression for the source-backed W-30 capture-to-pad audition path.
- Exercised capture commit, pad promotion, promoted audition, and source-backed preview projection in one flow.
- Asserted that the final W-30 preview carries non-empty source-window samples.

## Verification

- `cargo test -p riotbox-app captured_source_window_promotes_to_pad_and_auditions_source_preview`
- `git diff --check`
- `just ci`
- Branch diff reviewed with the `code-review` skill
- GitHub Actions `rust-ci`

## Notes

- Regression-only slice; no new sequencing UI, audio renderer behavior, TUI control, or generated audio artifact convention changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
