# `RIOTBOX-1315` P023: Move release-demo human-review queue fixture contract out of Justfile

- Ticket: `RIOTBOX-1315`
- Title: `P023: Move release-demo human-review queue fixture contract out of Justfile`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1315/p023-move-release-demo-human-review-queue-fixture-contract-out-of`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1315-p023-move-release-demo-human-review-queue-fixture-contract`
- Linear branch: `feature/riotbox-1315-p023-move-release-demo-human-review-queue-fixture-contract`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1289 (https://github.com/marang/riotbox/pull/1289)`
- Merge commit: `d6164836a190035ca5f47da1d6b7c692af0a7c9d`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_release_demo_human_review_queue.py scripts/release_demo_human_review_queue_fixtures.py; just release-demo-human-review-queue-fixtures; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1315-readiness-split; git diff --check; GitHub rust-ci pass on PR #1289`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

The release-demo human-review queue fixture contract was hidden in a large Justfile jq block, making P023 workflow proof hard to review.

## What Shipped

- Moved queue fixture contract and mutation checks into scripts/release_demo_human_review_queue_fixtures.py, added --mutation-fixtures, shortened Justfile, and documented the extraction in the roadmap.

## Notes

- Broad audio QA intentionally skipped; this was a contract/validator extraction and the affected downstream readiness consumer was validated directly.
