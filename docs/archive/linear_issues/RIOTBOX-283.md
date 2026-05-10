# `RIOTBOX-283` Sketch offline WAV render review path for fixture cases

- Ticket: `RIOTBOX-283`
- Title: `Sketch offline WAV render review path for fixture cases`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-283/sketch-offline-wav-render-review-path-for-fixture-cases`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-283-sketch-offline-wav-render-review-path-for-fixture-cases`
- Linear branch: `feature/riotbox-283-sketch-offline-wav-render-review-path-for-fixture-cases`
- PR: `#273`
- Merge commit: `f8d3394`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-284`

## Why This Ticket Existed

The audio QA workflow calls for deterministic offline WAV render support after widening buffer metrics. Riotbox needed a first bounded local render path that produces a reviewable artifact without pretending a full listening-pack or baseline-comparison system already exists.

## What Shipped

- Added a public W-30 preview offline render helper and basic signal metrics.
- Added `w30_preview_render`, a local CLI that writes one deterministic source-window smoke WAV plus sibling Markdown metrics.
- Documented the helper as local-only current QA status and added the command to the agent command shortlist.

## Verification

- `cargo test -p riotbox-audio offline_w30_preview_render_produces_reviewable_metrics`
- `cargo test -p riotbox-audio --bin w30_preview_render`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --out /tmp/riotbox-w30-preview-smoke/candidate.wav --duration-seconds 0.1`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- First local render helper only; no full fixture parser, listening-pack manifest, baseline-vs-candidate engine, or CI artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
