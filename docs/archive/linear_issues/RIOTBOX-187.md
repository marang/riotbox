# `RIOTBOX-187` Add a bounded source-audio decode cache for future W-30 raw playback

- Ticket: `RIOTBOX-187`
- Title: `Add a bounded source-audio decode cache for future W-30 raw playback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-187/add-a-bounded-source-audio-decode-cache-for-future-w-30-raw-playback`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-187-source-audio-decode-cache`
- Linear branch: `feature/riotbox-187-add-a-bounded-source-audio-decode-cache-for-future-w-30-raw`
- PR: `#177`
- Merge commit: `5127fd8`
- Labels: `Audio`, `Core`, `Spike`
- Follow-ups: `RIOTBOX-188`

## Why This Ticket Existed

The raw-capture audition seam was replay-safe and audible, but it still used a synthetic preview profile rather than captured source waveform data. Riotbox needed the next non-realtime audio seam before W-30 raw playback can become real without violating callback boundaries.

## What Shipped

- Added `SourceAudioCache` in `riotbox-audio` for non-realtime PCM16 WAV decoding into normalized interleaved `f32` samples.
- Added bounded frame/window access for later W-30 raw-capture preview playback.
- Added generated-WAV tests for source loading, window clamping, invalid WAV rejection, and unsupported bit-depth rejection.
- Documented the source-audio cache seam in `docs/specs/audio_core_spec.md`.

## Verification

- `cargo test -p riotbox-audio source_audio`
- `cargo test -p riotbox-audio`
- `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This intentionally does not wire cached waveform data into W-30 playback yet. It establishes the non-realtime decode/cache boundary needed by the next capture-window and raw-playback slices.
