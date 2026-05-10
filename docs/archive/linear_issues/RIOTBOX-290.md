# `RIOTBOX-290` Allow W-30 smoke render from a source WAV window

- Ticket: `RIOTBOX-290`
- Title: `Allow W-30 smoke render from a source WAV window`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-290/allow-w-30-smoke-render-from-a-source-wav-window`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-290-allow-w-30-smoke-render-from-a-source-wav-window`
- Linear branch: `feature/riotbox-290-allow-w-30-smoke-render-from-a-source-wav-window`
- PR: `#280`
- Merge commit: `2aec5e7`
- Labels: `benchmark`
- Follow-ups: `RIOTBOX-291`

## Why This Ticket Existed

The W-30 smoke QA helper rendered a deterministic synthetic source-window sample array. The next product-relevant slice was to let that same offline smoke path render a bounded window from a real PCM WAV source when provided.

## What Shipped

- Added optional `--source`, `--source-start-seconds`, and `--source-duration-seconds` flags to `w30_preview_render`.
- Used the existing non-realtime `SourceAudioCache` path to build the W-30 preview source window.
- Preserved the deterministic synthetic source-window default and documented the source-backed smoke command.

## Verification

- `cargo test -p riotbox-audio --bin w30_preview_render`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role candidate --duration-seconds 0.1 --source 'data/test_audio/examples/DH_BeatC_120-01.wav' --source-start-seconds 0.0 --source-duration-seconds 0.25`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role baseline --duration-seconds 0.1 --source 'data/test_audio/examples/DH_BeatC_120-01.wav' --source-start-seconds 0.0 --source-duration-seconds 0.25`
- `cargo run -p riotbox-audio --bin w30_preview_compare -- --date 2026-04-26`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Source-backed local QA helper only; no broad fixture-pack runner, live TUI playback behavior, baseline promotion workflow, or non-PCM WAV support changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
