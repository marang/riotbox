# `RIOTBOX-802` Improve W-30 chop articulation for musical showcase candidates

- Ticket: `RIOTBOX-802`
- Title: `Improve W-30 chop articulation for musical showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-802/improve-w-30-chop-articulation-for-musical-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-802-w30-chop-articulation`
- Linear branch: `feature/riotbox-802-improve-w-30-chop-articulation-for-musical-showcase`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#797 (https://github.com/marang/riotbox/pull/797)`
- Merge commit: `53b1bf08cef942eb4c8562fb4e5915d707ec686e`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-802-w30-tests-4.log cargo test -p riotbox-audio --bin feral_grid_pack w30_source_chop -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-802-feral-grid-tests-3.log cargo test -p riotbox-audio --bin feral_grid_pack`; `scripts/run_compact.sh /tmp/riotbox-802-syncopated-smoke-3.log just syncopated-source-showcase-smoke`; `scripts/run_compact.sh /tmp/riotbox-802-showcase-4.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-802-showcase-4 local-riotbox-802-4 4.0 4`; `scripts/run_compact.sh /tmp/riotbox-802-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-802-clippy-audio.log cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-802-fmt.log cargo fmt --check`; `git diff --check`; `GitHub Actions Rust CI run 1930 passed on ca0a02ed75e26e529740b2ee1e04ccf4285ab4ce`
- Docs touched: `docs/reviews/w30_chop_articulation_showcase_review_2026-05-14.md`, `docs/README.md`
- Follow-ups: `RIOTBOX-803`

## Why This Ticket Existed

The representative showcase W-30 source chop still felt too much like a flat technical audition after the first musical-quality gate landed.

## What Shipped

- Added attack/decay articulation, transient emphasis, tighter normalization, W-30 body/tail manifest metrics, a unit regression, and a review note while preserving source-diversity and audio-QA gates.

## Notes

- The first naive articulation attempt failed source-diversity on hat_cut_pressure vs tonal_hook_chop; the final implementation fixed this by preserving source-specific gain identity instead of relaxing gates.
