# `RIOTBOX-810` Add MC-202 source-grid alignment proof to representative showcase

- Ticket: `RIOTBOX-810`
- Title: `Add MC-202 source-grid alignment proof to representative showcase`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-810/add-mc-202-source-grid-alignment-proof-to-representative-showcase`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-810-mc202-source-grid-proof`
- Linear branch: `feature/riotbox-810-add-mc-202-source-grid-alignment-proof-to-representative`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#805 (https://github.com/marang/riotbox/pull/805)`
- Merge commit: `3f1c456dcb3f1edbbd5ba1c3b3f3844ab37681da`
- Verification: `cargo fmt --check`; `git diff --check`; `python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py`; `cargo test -p riotbox-audio --bin feral_grid_pack`; `just representative-source-showcase-musical-quality-fixtures`; `scripts/generate_representative_source_showcase.sh /tmp/riotbox-810-showcase local-riotbox-810 4.0 4`; `just syncopated-source-showcase-smoke`; `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `just audio-qa-ci`; `just ci`; GitHub Rust CI run `1954`
- Docs touched: `docs/benchmarks/representative_source_showcase_2026-05-07.md`
- Follow-ups: `None`

## Why This Ticket Existed

The P013 representative showcase needed MC-202 lane-specific source-grid proof so a drifting bass-pressure stem could not pass behind stronger aligned TR-909, W-30, or full-mix peaks.

## What Shipped

- Added MC-202 source-grid alignment metrics to feral_grid_pack reports/manifests, added a quiet offline MC-202 root-anchor component, tightened the musical-quality validator with a targeted drift fixture, updated showcase docs, and merged PR #805.

## Notes

- Linear deletion was not performed because `LINEAR_API_TOKEN` was not present in this environment.
