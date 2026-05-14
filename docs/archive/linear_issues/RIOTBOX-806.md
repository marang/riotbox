# `RIOTBOX-806` Add MC-202 bass-pressure lane to representative showcase candidates

- Ticket: `RIOTBOX-806`
- Title: `Add MC-202 bass-pressure lane to representative showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-806/add-mc-202-bass-pressure-lane-to-representative-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-806-mc202-bass-pressure-showcase`
- Linear branch: `feature/riotbox-806-add-mc-202-bass-pressure-lane-to-representative-showcase`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#801 (https://github.com/marang/riotbox/pull/801)`
- Merge commit: `21eedda198292bf4e9f1f242ece1be1d68ea84f7`
- Deleted from Linear: `2026-05-14`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack`; `just representative-source-showcase-musical-quality-fixtures`; `scripts/generate_representative_source_showcase.sh /tmp/riotbox-806-showcase local-riotbox-806 4.0 4`; `just syncopated-source-showcase-smoke`; `cargo fmt --check`; `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `just audio-qa-ci`; GitHub Rust CI #1942 passed.
- Docs touched: `docs/reviews/mc202_bass_pressure_showcase_review_2026-05-14.md`, `docs/benchmarks/representative_source_showcase_2026-05-07.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

Add an audible MC-202 bass-pressure lane to the representative Feral showcase so P013 all-lane musical depth is not limited to TR-909 plus W-30.

## What Shipped

- Rendered stems/03_mc202_bass_pressure.wav through the existing MC-202 offline render state, mixed it into source-first/generated-support renders, added manifest/report metrics, validator thresholds, fixtures, smoke gates, docs, and review proof.

## Notes

- None
