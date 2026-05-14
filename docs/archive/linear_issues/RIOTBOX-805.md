# `RIOTBOX-805` Add TR-909 kick-pressure variation for showcase candidates

- Ticket: `RIOTBOX-805`
- Title: `Add TR-909 kick-pressure variation for showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-805/add-tr-909-kick-pressure-variation-for-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-805-tr909-kick-pressure-variation`
- Linear branch: `feature/riotbox-805-add-tr-909-kick-pressure-variation-for-showcase-candidates`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#800 (https://github.com/marang/riotbox/pull/800)`
- Merge commit: `73548478a530e3960adf37ae54cd358c4c216400`
- Deleted from Linear: `2026-05-14`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack`, `just representative-source-showcase-musical-quality-fixtures`, `python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_source_showcase_diversity.py`, `scripts/generate_representative_source_showcase.sh /tmp/riotbox-805-showcase local-riotbox-805 4.0 4`, `just syncopated-source-showcase-smoke`, `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `just audio-qa-ci`, GitHub Actions Rust CI #1939`
- Docs touched: `docs/reviews/tr909_kick_pressure_showcase_review_2026-05-14.md`, `docs/benchmarks/representative_source_showcase_2026-05-07.md`
- Follow-ups: `None`

## Why This Ticket Existed

P013 needed generated TR-909 support to add measurable kick/body pressure in the representative showcase after W-30 articulation and slice-choice depth landed.

## What Shipped

- Added a bounded source-aware TR-909 kick-pressure layer inside feral_grid_pack, exposed pressure proof metrics in manifest/report output, tightened musical-quality and syncopated smoke gates against decorative pressure, and recorded representative showcase evidence including the first too-uniform attempt.

## Notes

- None
