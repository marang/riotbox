# `RIOTBOX-1328` P023: Add master-bus gain staging, clip/DC metrics, and limiter policy

- Ticket: `RIOTBOX-1328`
- Title: `P023: Add master-bus gain staging, clip/DC metrics, and limiter policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1328/p023-add-master-bus-gain-staging-clipdc-metrics-and-limiter-policy`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1328-p023-add-master-bus-gain-staging-clipdc-metrics-and-limiter`
- Linear branch: `feature/riotbox-1328-p023-add-master-bus-gain-staging-clipdc-metrics-and-limiter`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1302 (https://github.com/marang/riotbox/pull/1302)`
- Merge commit: `8ebcad8e502898583c4dc638ef7bd34b38ee2c2d`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio; cargo test -p riotbox-app; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings; GitHub rust-ci success`
- Docs touched: `docs/specs/audio_core_spec.md`
- Follow-ups: `RIOTBOX-1339 tracks the shared master-bus soft limiter implementation without masking weak-output gates.`

## Why This Ticket Existed

Make Riotbox audio reports expose clipping and remaining headroom before file-output clamping can hide level problems.

## What Shipped

- Extended OfflineAudioMetrics and listening manifests with clip_count, near_clip_count, and headroom_to_full_scale; surfaced those metrics in W-30 preview, feral before/after, and feral grid reports; documented the current no-global-limiter policy.

## Notes

- This ticket intentionally did not add a product-path limiter. The limiter remains a separate shared master-bus implementation so weak-output, source-character, and fallback-collapse gates stay honest.
