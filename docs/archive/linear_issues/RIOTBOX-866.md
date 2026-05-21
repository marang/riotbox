# `RIOTBOX-866` Pin Beat08 auto-BPM Feral grid Source Timing path

- Ticket: `RIOTBOX-866`
- Title: `Pin Beat08 auto-BPM Feral grid Source Timing path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-866/pin-beat08-auto-bpm-feral-grid-source-timing-path`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-866-pin-beat08-auto-bpm-feral-grid-source-timing-path`
- Linear branch: `feature/riotbox-866-pin-beat08-auto-bpm-feral-grid-source-timing-path`
- Assignee: `Markus`
- Labels: `benchmark`, `ux`
- PR: `#860 (https://github.com/marang/riotbox/pull/860)`
- Merge commit: `0d6979df1658ddf4fa75ae8e9e0aaa17cf062327`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox866-beat08-proof-final.log just beat08-auto-feral-grid-proof local-beat08-feral-grid-auto-proof; GitHub Actions Rust CI success on PR #860`
- Docs touched: `Justfile`, `docs/benchmarks/README.md`, `docs/benchmarks/beat08_auto_feral_grid_source_timing_2026-05-21.md`, `scripts/validate_auto_feral_grid_source_timing_pack.sh`
- Follow-ups: `None`

## Why This Ticket Existed

Beat08 is the primary documented source for first timing and queue/commit workflows, and it was the remaining Recipe 15 auto variant without a pinned local Source Timing Feral-grid proof.

## What Shipped

- Extended the local auto Feral-grid Source Timing proof helper with Beat08, added the Beat08 proof target, and recorded the Beat08 auto-BPM benchmark evidence.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, or realtime audio behavior changed. Local proof result: bpm=128.397, TR-909 hit_ratio=1.000, MC-202 hit_ratio=1.000, W-30 hit_ratio=0.750, mix hit_ratio=0.969.
