# `RIOTBOX-865` Pin DH_BeatC auto-BPM Feral grid Source Timing path

- Ticket: `RIOTBOX-865`
- Title: `Pin DH_BeatC auto-BPM Feral grid Source Timing path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-865/pin-dh-beatc-auto-bpm-feral-grid-source-timing-path`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-865-pin-dh_beatc-auto-bpm-feral-grid-source-timing-path`
- Linear branch: `feature/riotbox-865-pin-dh_beatc-auto-bpm-feral-grid-source-timing-path`
- Assignee: `Markus`
- Labels: `benchmark`, `ux`
- PR: `#859 (https://github.com/marang/riotbox/pull/859)`
- Merge commit: `a6feb8f85c7a4179966a96b9d4f1e204f156b58f`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox865-beat03-proof-final.log just beat03-auto-feral-grid-proof local-beat03-feral-grid-auto-proof; scripts/run_compact.sh /tmp/riotbox865-dh-beatc-proof-final.log just dh-beatc-auto-feral-grid-proof local-dh-beatc-feral-grid-auto-proof; GitHub Actions Rust CI success on PR #859`
- Docs touched: `Justfile`, `docs/benchmarks/README.md`, `docs/benchmarks/dh_beatc_auto_feral_grid_source_timing_2026-05-21.md`, `scripts/validate_auto_feral_grid_source_timing_pack.sh`, `scripts/validate_beat03_auto_feral_grid_pack.sh`
- Follow-ups: `None`

## Why This Ticket Existed

DH_BeatC was listed as an auto Feral-grid Recipe 15 variant and showed the same cautious Source Timing path as Beat03, but it did not have a pinned local proof or benchmark note.

## What Shipped

- Generalized the local auto Feral-grid Source Timing proof helper, kept the Beat03 proof entrypoint working, added a DH_BeatC proof target, and recorded the DH_BeatC auto-BPM Source Timing benchmark.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, or realtime audio behavior changed. Local proof result: bpm=120.185, TR-909 hit_ratio=1.000, MC-202 hit_ratio=1.000, W-30 hit_ratio=0.750, mix hit_ratio=0.969.
