# `RIOTBOX-1218` Add negative fixtures for rebuild-only professional-output proof

- Ticket: `RIOTBOX-1218`
- Title: `Add negative fixtures for rebuild-only professional-output proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1218/add-negative-fixtures-for-rebuild-only-professional-output-proof`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1218-add-negative-fixtures-for-rebuild-only-professional-output`
- Linear branch: `feature/riotbox-1218-add-negative-fixtures-for-rebuild-only-professional-output`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1193 (https://github.com/marang/riotbox/pull/1193)`
- Merge commit: `e4894eb09fbf972797b069fc9a2b2b4c64f61aa3`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1218-dense-smoke-review; scripts/run_compact.sh /tmp/riotbox-1218-audio-qa-ci-review.log just audio-qa-ci; scripts/run_compact.sh /tmp/riotbox-1218-just-ci-review.log just ci; GitHub Actions rust-ci passed for PR #1193`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`
- Follow-ups: `RIOTBOX-1219 pad/noise source-family policy; RIOTBOX-1220 bad-timing cautious arrangement policy`

## Why This Ticket Existed

Software: dense-break report validation now recomputes rebuild-only guard failures and rejects stale pass/failure-code state. Musician: diagnostic demos cannot pass when rebuild-only output is silent, source-masked, or unchanged by source-layer toggling.

## What Shipped

- Added --validate-report for dense-break performance reports, CI-safe negative report mutations for silent rebuild-only/source masking/source-toggle collapse, and benchmark documentation for the negative fixture boundary.

## Notes

- None
