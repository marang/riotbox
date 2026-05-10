# `RIOTBOX-273` Surface W-30 hit source readiness in Capture handoff

- Ticket: `RIOTBOX-273`
- Title: `Surface W-30 hit source readiness in Capture handoff`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-273/surface-w-30-hit-source-readiness-in-capture-handoff`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-273-surface-w-30-hit-source-readiness-in-capture-handoff`
- Linear branch: `feature/riotbox-273-surface-w-30-hit-source-readiness-in-capture-handoff`
- PR: `#263`
- Merge commit: `9b14eea`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-274`

## Why This Ticket Existed

The Capture flow already told users how to audition, promote, and hit, but the promoted `[w] hit` handoff did not say whether the path was source-backed or the safe fallback preview. That made the W-30 learning path harder to interpret.

## What Shipped

- Added compact `src` / `fallback` readiness to Capture handoff copy where `[w] hit` or `[p]->[w]` is suggested.
- Added source-backed W-30 target regression coverage and updated the W-30 fixture baseline.
- Updated `docs/jam_recipes.md` to explain the new handoff cue.

## Verification

- `cargo test -p riotbox-app renders_capture_handoff_source_readiness_for_w30_targets`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_raw_capture_audition_cue`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_capture_context`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI/DOCS slice only; no audio rendering, keymap, or Capture layout behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
