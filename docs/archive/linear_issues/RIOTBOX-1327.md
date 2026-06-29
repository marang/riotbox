# `RIOTBOX-1327` P023: Design coherent render-state snapshots for realtime audio

- Ticket: `RIOTBOX-1327`
- Title: `P023: Design coherent render-state snapshots for realtime audio`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1327/p023-design-coherent-render-state-snapshots-for-realtime-audio`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1327-p023-design-coherent-render-state-snapshots-for-realtime`
- Linear branch: `feature/riotbox-1327-p023-design-coherent-render-state-snapshots-for-realtime`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1301 (https://github.com/marang/riotbox/pull/1301)`
- Merge commit: `032307c62a398182d9d075623c775cc3b7703226`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio; cargo test -p riotbox-app; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings; GitHub rust-ci success`
- Docs touched: `docs/specs/audio_core_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prevent the realtime callback from reading mixed old/new lane control state while preserving callback-safe handoff behavior.

## What Shipped

- Added bounded coherent snapshot helpers and revisioned control handoffs for transport, TR-909, MC-202, W-30 preview/resample, and Source Monitor callback state.

## Notes

- The callback reuses the last complete snapshot when a writer update is in flight; no blocking I/O, allocation, or musical fallback was added to the audio path.
