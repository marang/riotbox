# `RIOTBOX-804` Add W-30 source slice-choice variation for showcase candidates

- Ticket: `RIOTBOX-804`
- Title: `Add W-30 source slice-choice variation for showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-804/add-w-30-source-slice-choice-variation-for-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-804-w30-slice-choice-variation`
- Linear branch: `feature/riotbox-804-add-w-30-source-slice-choice-variation-for-showcase`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#799 (https://github.com/marang/riotbox/pull/799)`
- Merge commit: `1f60c39b0e1752340d0e199115aa7fbc2df93465`
- Deleted from Linear: `2026-05-14`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack`, `just representative-source-showcase-musical-quality-fixtures`, `scripts/generate_representative_source_showcase.sh /tmp/riotbox-804-showcase local-riotbox-804 4.0 4`, `just syncopated-source-showcase-smoke`, `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `just audio-qa-ci`, GitHub Actions Rust CI #1936`
- Docs touched: `docs/reviews/w30_slice_choice_showcase_review_2026-05-14.md`, `docs/benchmarks/representative_source_showcase_2026-05-07.md`
- Follow-ups: `None`

## Why This Ticket Existed

P013 needed the W-30 source-backed showcase lane to vary not only trigger timing but also which analyzed source offsets are read, so repeated chops do not collapse into one static audition.

## What Shipped

- Added deterministic W-30 source slice-choice planning inside feral_grid_pack, exposed unique-offset/span metrics in manifest and report output, tightened musical-quality and syncopated smoke gates against static slice-choice collapse, and recorded representative showcase evidence.

## Notes

- None
