# `RIOTBOX-291` Support PCM24 WAV source loading for W-30 smoke path

- Ticket: `RIOTBOX-291`
- Title: `Support PCM24 WAV source loading for W-30 smoke path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-291/support-pcm24-wav-source-loading-for-w-30-smoke-path`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-291-support-pcm24-wav-source-loading-for-w-30-smoke-path`
- Linear branch: `feature/riotbox-291-support-pcm24-wav-source-loading-for-w-30-smoke-path`
- PR: `#281`
- Merge commit: `9d8a724`
- Labels: `benchmark`
- Follow-ups: `RIOTBOX-292`

## Why This Ticket Existed

`RIOTBOX-290` made the smoke renderer accept real WAV source windows, but several musician-facing example files are 24-bit PCM. The non-realtime source loader needed bounded PCM24 support so W-30 smoke QA can use those examples without adding a broad importer.

## What Shipped

- Extended the non-realtime WAV decoder to support Microsoft PCM 24-bit samples.
- Kept PCM16 behavior covered and kept unsupported bit-depth rejection explicit.
- Updated audio specs and the W-30 smoke pack docs to reflect PCM16/PCM24 source-window input.

## Verification

- `cargo test -p riotbox-audio source_audio::tests`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role baseline --duration-seconds 0.1 --source 'data/test_audio/examples/Beat03_130BPM(Full).wav' --source-start-seconds 0.0 --source-duration-seconds 0.25`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role candidate --duration-seconds 0.1 --source 'data/test_audio/examples/Beat03_130BPM(Full).wav' --source-start-seconds 0.0 --source-duration-seconds 0.25`
- `cargo run -p riotbox-audio --bin w30_preview_compare -- --date 2026-04-26`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Non-realtime decoder support only; no float WAV support, compressed WAV support, realtime audio-path decoding, or broad sample-library importer changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
