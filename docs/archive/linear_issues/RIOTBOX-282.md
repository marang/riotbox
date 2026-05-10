# `RIOTBOX-282` Widen W-30 preview fixture signal metrics

- Ticket: `RIOTBOX-282`
- Title: `Widen W-30 preview fixture signal metrics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-282/widen-w-30-preview-fixture-signal-metrics`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-282-widen-w-30-preview-fixture-signal-metrics`
- Linear branch: `feature/riotbox-282-widen-w-30-preview-fixture-signal-metrics`
- PR: `#272`
- Merge commit: `30d53d1`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-283`

## Why This Ticket Existed

The audio QA workflow calls for widening signal metrics on existing buffer regression fixtures before adding offline WAV review packs. W-30 preview fixtures had active-sample and peak checks, but source-window cases also needed a bounded signal-shape check so future sampler preview drift is easier to catch.

## What Shipped

- Made W-30 preview fixture regressions enforce optional `min_sum` / `max_sum` ranges.
- Added optional `min_rms` / `max_rms` fixture expectations.
- Pinned RMS for the source-window preview fixture and updated the audio QA workflow status.

## Verification

- `cargo test -p riotbox-audio fixture_backed_w30_preview_audio_regressions_hold`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- QA harness slice only; no DSP behavior, offline WAV render path, or listening-pack workflow changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
