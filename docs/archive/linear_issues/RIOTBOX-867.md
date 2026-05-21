# `RIOTBOX-867` Pin Beat20 ambiguous auto-BPM Feral grid fallback path

- Ticket: `RIOTBOX-867`
- Title: `Pin Beat20 ambiguous auto-BPM Feral grid fallback path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-867/pin-beat20-ambiguous-auto-bpm-feral-grid-fallback-path`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-867-pin-beat20-ambiguous-auto-bpm-feral-grid-fallback-path`
- Linear branch: `feature/riotbox-867-pin-beat20-ambiguous-auto-bpm-feral-grid-fallback-path`
- Assignee: `Markus`
- Labels: `benchmark`, `ux`
- PR: `#861 (https://github.com/marang/riotbox/pull/861)`
- Merge commit: `42954c4b808c9bb5d562df5e4e969eae9eef4f8d`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox867-beat20-fallback-proof-final.log just beat20-auto-feral-grid-fallback-proof local-beat20-feral-grid-auto-fallback-proof; GitHub Actions Rust CI success on PR #861`
- Docs touched: `Justfile`, `docs/benchmarks/README.md`, `docs/benchmarks/beat20_auto_feral_grid_fallback_2026-05-21.md`, `scripts/validate_auto_feral_grid_source_timing_pack.sh`
- Follow-ups: `None`

## Why This Ticket Existed

Beat20 has useful BPM and beat evidence after RIOTBOX-862, but ambiguous downbeat phases must keep Feral-grid auto mode from treating Source Timing as grid-ready.

## What Shipped

- Added a Beat20 ambiguous/manual-only profile to the local auto Feral-grid proof helper, added the Beat20 fallback proof target, and recorded the fallback benchmark evidence.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, or realtime audio behavior changed. Local proof result: grid_source=static_default, reason=source_timing_requires_manual_confirm, bpm=128.397, TR-909 hit_ratio=1.000, MC-202 hit_ratio=1.000, W-30 hit_ratio=0.750, mix hit_ratio=1.000.
