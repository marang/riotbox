# `RIOTBOX-1326` P023: Audit and harden the realtime audio callback hot path

- Ticket: `RIOTBOX-1326`
- Title: `P023: Audit and harden the realtime audio callback hot path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1326/p023-audit-and-harden-the-realtime-audio-callback-hot-path`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1326-p023-audit-and-harden-the-realtime-audio-callback-hot-path`
- Linear branch: `feature/riotbox-1326-p023-audit-and-harden-the-realtime-audio-callback-hot-path`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1300 (https://github.com/marang/riotbox/pull/1300)`
- Merge commit: `8ce8705fc2bd1125c73b8fc66de3b15098b8fc43`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio; cargo test -p riotbox-app; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings; GitHub rust-ci success`
- Docs touched: `docs/reviews/realtime_audio_callback_hot_path_audit_2026-06-29.md`
- Follow-ups: `RIOTBOX-1327 is the coherent render-state snapshot follow-up; RIOTBOX-1338 was closed as duplicate of RIOTBOX-1327.`

## Why This Ticket Existed

P023 needed the realtime audio callback audited and hardened before more device-facing work, especially around callback allocation, source-cache work, and visible degraded state.

## What Shipped

- Removed normal callback-side scratch buffer growth, added visible scratch-overflow telemetry through runtime/observer warnings, kept source-monitor source cache borrowed in callback snapshots, and documented the callback hot path.

## Notes

- No musical behavior changes; oversized backend buffers are explicitly degraded with silence for that buffer plus health telemetry rather than hidden allocation in the audio thread.
