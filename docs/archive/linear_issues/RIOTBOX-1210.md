# `RIOTBOX-1210` Capture 20/10 sound-product future ideas

- Ticket: `RIOTBOX-1210`
- Title: `Capture 20/10 sound-product future ideas`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1210/capture-2010-sound-product-future-ideas`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1210-capture-2010-sound-product-future-ideas`
- Linear branch: `feature/riotbox-1210-capture-2010-sound-product-future-ideas`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1186 (https://github.com/marang/riotbox/pull/1186)`
- Merge commit: `a61d79843f2ce2ee7cce36193cfc249ff0c1d2fd`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/validate_sound_product_2010_future_ideas.py; just sound-product-2010-future-ideas-fixtures; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed on PR #1186.`
- Docs touched: `docs/specs/sound_product_2010_future_ideas_spec.md; docs/README.md; docs/benchmarks/README.md; docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Keep ambitious 20/10 sound-product ideas visible without letting them obscure or block the 10/10 release path.

## What Shipped

- Added a 20/10 sound-product future-ideas spec, machine-checkable idea list with seven initial directions, release-blocking negative fixture, validator, audio-qa-ci wiring, and docs/roadmap/benchmark indexes.

## Notes

- Every 20/10 idea remains release_blocking false and requires normal Linear promotion before implementation.
