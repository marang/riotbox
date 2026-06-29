# `RIOTBOX-1329` P023: Add offline/realtime render parity seam

- Ticket: `RIOTBOX-1329`
- Title: `P023: Add offline/realtime render parity seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1329/p023-add-offlinerealtime-render-parity-seam`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1329-p023-add-offlinerealtime-render-parity-seam`
- Linear branch: `feature/riotbox-1329-p023-add-offlinerealtime-render-parity-seam`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1303 (https://github.com/marang/riotbox/pull/1303)`
- Merge commit: `c5f29d266df69f07aebab32f489e15fecb18b74e`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo test -p riotbox-audio runtime_mix_ -- --nocapture; cargo test -p riotbox-audio; cargo test -p riotbox-app; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings; cargo fmt --check; GitHub rust-ci success`
- Docs touched: `docs/specs/audio_core_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Catch runtime audio drift between deterministic offline QA renders and callback-sized realtime simulation before it reaches demos or live playback.

## What Shipped

- Added RuntimeMixRenderPlan plus full-block and callback-blocked runtime mix render helpers covering TR-909, MC-202, W-30 preview/resample, and Source Monitor policy; added parity and Riotbox-only default tests.

## Notes

- The seam is CI-safe and does not involve CPAL/device output. It proves buffer-boundary parity for a covered runtime mix state without a broad audio engine rewrite.
