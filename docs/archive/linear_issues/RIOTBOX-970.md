# `RIOTBOX-970` Route source monitor mode through audio runtime policy

- Ticket: `RIOTBOX-970`
- Title: `Route source monitor mode through audio runtime policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-970/route-source-monitor-mode-through-audio-runtime-policy`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-23`
- Branch: `feature/riotbox-970-route-source-monitor-mode-through-audio-runtime-policy`
- Linear branch: `feature/riotbox-970-route-source-monitor-mode-through-audio-runtime-policy`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#962 (https://github.com/marang/riotbox/pull/962)`
- Merge commit: `dd5af1b34db3fc745314437774985ea211c8c378`
- Deleted from Linear: `2026-05-23`
- Verification: `cargo test -p riotbox-audio runtime::tests::source_monitor -- --nocapture; cargo test -p riotbox-audio runtime::tests::render_mix_buffer_includes_live_mc202_bass_seam -- --nocapture; cargo test -p riotbox-audio runtime::tests:: -- --nocapture; cargo test -p riotbox-app jam_app::tests::runtime_view_updates_from_audio_and_sidecar_state -- --nocapture; cargo test -p riotbox-app jam_app::tests::source_monitor -- --nocapture; cargo test -p riotbox-audio; cargo test -p riotbox-app; git diff --check; just ci; post-rebase just ci; GitHub Actions Rust CI #2438`
- Docs touched: `none`
- Follow-ups: `Next source workflow slices should add Source Map rendering, seek bar/phrase navigation, grid confirmation, capture-length UI, and sample-rate conversion for mismatched source/device rates.`

## Why This Ticket Existed

Make the source/blend/riotbox monitor contract audible through the realtime audio runtime instead of observer-only state.

## What Shipped

- Added a realtime-safe source monitor audio policy with immutable preloaded Source PCM, atomic mode/gain updates, source/blend/riotbox/fallback routing, Jam runtime route diagnostics, observer route output, and audio/app regression coverage.

## Notes

- No callback file I/O or analysis work. Mismatched source/audio-device sample rates fall back to Riotbox output; resampling is a follow-up.
