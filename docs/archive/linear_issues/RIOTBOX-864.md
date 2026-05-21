# `RIOTBOX-864` Document and pin Beat03 auto-BPM Feral grid Source Timing path

- Ticket: `RIOTBOX-864`
- Title: `Document and pin Beat03 auto-BPM Feral grid Source Timing path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-864/document-and-pin-beat03-auto-bpm-feral-grid-source-timing-path`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-864-document-and-pin-beat03-auto-bpm-feral-grid-source-timing`
- Linear branch: `feature/riotbox-864-document-and-pin-beat03-auto-bpm-feral-grid-source-timing`
- Assignee: `Markus`
- Labels: `benchmark`, `ux`
- PR: `#858 (https://github.com/marang/riotbox/pull/858)`
- Merge commit: `2bed7bad96daf4427e173c4c64143d038b113731`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox864-beat03-auto-proof-rerun.log just beat03-auto-feral-grid-proof local-beat03-feral-grid-auto-proof; GitHub Actions Rust CI success on PR #858`
- Docs touched: `Justfile`, `docs/jam_recipes.md`, `docs/benchmarks/README.md`, `docs/benchmarks/beat03_auto_feral_grid_source_timing_2026-05-21.md`, `scripts/validate_beat03_auto_feral_grid_pack.sh`
- Follow-ups: `None`

## Why This Ticket Existed

Recipe 15 still told musicians to use explicit 130 BPM for Beat03 even though the current Source Timing path now drives Beat03 auto-BPM through source_timing_needs_review_manual_confirm with useful output evidence.

## What Shipped

- Added a local Beat03 auto Feral-grid proof, documented the current Source Timing-driven auto-BPM path, and updated the recipe and benchmark index so Beat03 auto is the primary current path while still showing manual-confirm caution.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, or realtime audio behavior changed. Local proof result: bpm=130.285, TR-909 hit_ratio=1.000, MC-202 hit_ratio=1.000, W-30 hit_ratio=0.750, mix hit_ratio=0.969.
