# `RIOTBOX-807` Add MC-202 bass-pressure phrase variation to showcase candidates

- Ticket: `RIOTBOX-807`
- Title: `Add MC-202 bass-pressure phrase variation to showcase candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-807/add-mc-202-bass-pressure-phrase-variation-to-showcase-candidates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-807-mc202-bass-phrase-variation`
- Linear branch: `feature/riotbox-807-add-mc-202-bass-pressure-phrase-variation-to-showcase`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#802 (https://github.com/marang/riotbox/pull/802)`
- Merge commit: `749e10da87674ebbc9f5e57e6ac8b7702d2ffae0`
- Verification: `Rust CI run 1945 passed; local just ci, audio-qa-ci, clippy audio, representative showcase, syncopated smoke, musical-quality fixtures, fmt, py_compile, and git diff --check passed`
- Docs touched: `docs/reviews/mc202_bass_phrase_variation_showcase_review_2026-05-20.md; docs/benchmarks/representative_source_showcase_2026-05-07.md; docs/README.md`
- Follow-ups: `RIOTBOX-808`

## Why This Ticket Existed

RIOTBOX-806 made the MC-202 bass-pressure lane audible, but the representative showcase still needed phrase/bar movement proof so the lane could not pass as one repeated static support cell.

## What Shipped

- Added deterministic per-bar MC-202 bass-pressure phrase profiles, manifest/report proof fields, validator thresholds, fixture updates, and review evidence for the representative showcase.

## Notes

- Bounded offline feral_grid_pack showcase behavior only; no ActionCommand, Session, realtime mixer, live TUI, or JamAppState change.
- Linear deletion was not performed during archive generation because `LINEAR_API_TOKEN` was not present in the local environment.
