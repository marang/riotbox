# `RIOTBOX-292` Add source-backed W-30 smoke just targets

- Ticket: `RIOTBOX-292`
- Title: `Add source-backed W-30 smoke just targets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-292/add-source-backed-w-30-smoke-just-targets`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-292-add-source-backed-w-30-smoke-just-targets`
- Linear branch: `feature/riotbox-292-add-source-backed-w-30-smoke-just-targets`
- PR: `#282`
- Merge commit: `1df2789`
- Labels: `benchmark`
- Follow-ups: `RIOTBOX-293`

## Why This Ticket Existed

`RIOTBOX-290` added optional source-window input to `w30_preview_render`, and `RIOTBOX-291` added PCM24 support for common example WAV files. The Justfile still only exposed the synthetic W-30 smoke path, so users and agents had to retype long cargo commands to exercise real source material.

## What Shipped

- Added `w30-smoke-source-baseline`, `w30-smoke-source-candidate`, and `w30-smoke-source-qa` Justfile targets.
- Kept the existing synthetic smoke targets unchanged.
- Documented the source-backed wrapper in the W-30 preview smoke listening-pack guide.
- Added the source-backed smoke command to the repo agent command shortlist.

## Verification

- `just w30-smoke-source-qa 'data/test_audio/examples/Beat03_130BPM(Full).wav' 2026-04-26 0.0 0.25 0.1`
- `git diff --check`
- `just ci`
- Branch diff reviewed with the `code-review` skill
- GitHub Actions `rust-ci`

## Notes

- Workflow/docs wrapper slice only; no decoder behavior, TUI playback, pad sequencer behavior, or generated audio artifact policy changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
