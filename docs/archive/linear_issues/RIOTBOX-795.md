# `RIOTBOX-795` Fix source-timing beat-period sort comparator panic on syncopated onsets

- Ticket: `RIOTBOX-795`
- Title: `Fix source-timing beat-period sort comparator panic on syncopated onsets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-795/fix-source-timing-beat-period-sort-comparator-panic-on-syncopated`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-795-period-score-total-order`
- Linear branch: `feature/riotbox-795-fix-source-timing-beat-period-sort-comparator-panic-on`
- Assignee: `Markus`
- Labels: `Bug`, `benchmark`, `timing`
- PR: `#791 (https://github.com/marang/riotbox/pull/791)`
- Merge commit: `7d1bb067c5868ba968f88ff6a40b78d4213355ee`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-795-core-source-timing-tests.log cargo test -p riotbox-core source_timing_probe_bpm_candidates -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-795-audio-fixture-test.log cargo test -p riotbox-audio source_timing_probe::tests::source_timing_probe_candidate_fixture_seed_scores_pcm_wav_grid -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-795-cargo-test-after-ci-fix.log cargo test`; `scripts/run_compact.sh /tmp/riotbox-795-clippy-after-ci-fix.log cargo clippy --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-795-fmt-after-ci-fix.log cargo fmt --check`; `scripts/generate_representative_source_showcase.sh /tmp/riotbox-795-representative-source-showcase-default local-representative-source-showcase-795-default`; `GitHub Actions Rust CI run 1912 passed on 7e294eef272f44a213186a52c2791b7029d8246e`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Representative source showcase generation exposed a Source Timing beat-period sort panic on syncopated onset material because the previous fuzzy comparator violated Rust sort total-order requirements.

## What Shipped

- Replaced pairwise fuzzy comparator behavior with deterministic score buckets plus period/match/score tie-breakers, preserving near-tie musical period preference while guaranteeing transitive sorting; added direct regression tests for both total-order and near-tie 120-vs-240 BPM behavior.

## Notes

- First CI run correctly caught a harmonic-selection regression; the final branch includes full cargo test and clippy verification after the fix.
