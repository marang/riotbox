# `RIOTBOX-803` Add W-30 source-chop trigger variation for showcase candidates

- Ticket: `RIOTBOX-803`
- Title: `Add W-30 source-chop trigger variation for showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-803/add-w-30-source-chop-trigger-variation-for-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-803-w30-trigger-variation`
- Linear branch: `feature/riotbox-803-add-w-30-source-chop-trigger-variation-for-showcase`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#798 (https://github.com/marang/riotbox/pull/798)`
- Merge commit: `d32df7678deda0c12caa227ba5dee8b9a6bbe203`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-803-w30-tests.log cargo test -p riotbox-audio --bin feral_grid_pack w30_source_chop -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-803-feral-grid-tests.log cargo test -p riotbox-audio --bin feral_grid_pack`; `scripts/run_compact.sh /tmp/riotbox-803-fixtures.log just representative-source-showcase-musical-quality-fixtures`; `scripts/run_compact.sh /tmp/riotbox-803-pycompile.log python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_source_showcase_diversity.py`; `scripts/run_compact.sh /tmp/riotbox-803-showcase-5.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-803-showcase-5 local-riotbox-803-5 4.0 4`; `scripts/run_compact.sh /tmp/riotbox-803-syncopated-smoke.log just syncopated-source-showcase-smoke`; `scripts/run_compact.sh /tmp/riotbox-803-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-803-clippy-audio.log cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-803-fmt.log cargo fmt --check`; `git diff --check`; `GitHub Actions Rust CI run 1933 passed on db5ec5eb971c368a5e367f8516d7c32cccd40d93`
- Docs touched: `docs/reviews/w30_trigger_variation_showcase_review_2026-05-14.md`, `docs/benchmarks/representative_source_showcase_2026-05-07.md`
- Follow-ups: `RIOTBOX-804`

## Why This Ticket Existed

RIOTBOX-802 shaped the W-30 source chop itself, but the showcase still retriggered it with a simple static feel.

## What Shipped

- Added bounded Feral-grid W-30 trigger variation, manifest/report proof fields, musical-quality validator checks, static negative fixture coverage, syncopated smoke updates, and a review note with representative showcase metrics.

## Notes

- Selected musical candidate after the slice is tonal_hook_chop/head with full RMS 0.028983, low-band RMS 0.027200, support/source ratio 0.171260, W-30 offbeat triggers 4, distinct bar patterns 4, and max quantized offset 0.0 ms. The realtime W-30 callback, Jam action model, Session, and ActionCommand paths are unchanged.
