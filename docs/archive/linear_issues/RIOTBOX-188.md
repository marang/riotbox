# `RIOTBOX-188` Persist capture source windows for future raw W-30 playback

- Ticket: `RIOTBOX-188`
- Title: `Persist capture source windows for future raw W-30 playback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-188/persist-capture-source-windows-for-future-raw-w-30-playback`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-188-capture-source-windows`
- Linear branch: `feature/riotbox-188-persist-capture-source-windows-for-future-raw-w-30-playback`
- PR: `#178`
- Merge commit: `951b075`
- Labels: `Audio`, `Core`
- Follow-ups: `RIOTBOX-189`

## Why This Ticket Existed

`RIOTBOX-187` added non-realtime PCM16 source decoding, but direct captures still did not preserve the exact source range they represented. Before W-30 raw audition can play real waveform material, captures need explicit replay-safe window metadata instead of relying on UI state or action-history inference.

## What Shipped

- Added optional `CaptureRef.source_window` metadata with source id, start/end seconds, and start/end source frames.
- Derived direct capture windows from committed transport boundary timing and the active `SourceGraph`.
- Preserved source-window metadata through session save/load and inherited it through source-backed derived captures where appropriate.
- Added regression coverage for computed capture windows, persistence roundtrip, and legacy JSON sessions that do not contain `source_window`.
- Documented the optional `source_window` contract in `docs/specs/session_file_spec.md`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app committed_capture_actions_materialize_capture_refs`
- `cargo test -p riotbox-core legacy_capture_refs_without_source_window_still_load`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This intentionally does not change the audible W-30 preview path yet. It makes the session model carry enough source-window truth for `RIOTBOX-189` to resolve raw W-30 audition against decoded source audio.
