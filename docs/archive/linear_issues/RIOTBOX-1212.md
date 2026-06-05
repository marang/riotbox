# `RIOTBOX-1212` Make dense-break 8-bar Golden Path source-aware

- Ticket: `RIOTBOX-1212`
- Title: `Make dense-break 8-bar Golden Path source-aware`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1212/make-dense-break-8-bar-golden-path-source-aware`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1212-make-dense-break-8-bar-golden-path-source-aware`
- Linear branch: `feature/riotbox-1212-make-dense-break-8-bar-golden-path-source-aware`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1187 (https://github.com/marang/riotbox/pull/1187)`
- Merge commit: `0b88b3740bba628c98593f2a2ebdad1c8830d899`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1212-dense-break-smoke; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1212-professional-output-suite; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed on PR #1187.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move dense-break Golden Path from fixed diagnostic recipe toward visible source-aware pressure, stutter, and restore policy.

## What Shipped

- Added DenseBreakSourcePolicy derived from source low-band, high-band, and transient metrics; applied it to pressure, bass, stutter, restore, report proof, smoke gate, and benchmark docs while preserving diagnostic/unverified evidence boundaries.

## Notes

- None
