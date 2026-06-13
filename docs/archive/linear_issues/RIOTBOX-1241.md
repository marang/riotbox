# `RIOTBOX-1241` P023: Generate release-demo human review queue

- Ticket: `RIOTBOX-1241`
- Title: `P023: Generate release-demo human review queue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1241/p023-generate-release-demo-human-review-queue`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1241-p023-release-demo-human-review-queue`
- Linear branch: `feature/riotbox-1241-p023-generate-release-demo-human-review-queue`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1216 (https://github.com/marang/riotbox/pull/1216)`
- Merge commit: `78be75f6fa4e8e41deae60ecf6287a7502c942db`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_release_demo_human_review_queue.py scripts/validate_source_family_release_demo_coverage.py; just release-demo-human-review-queue-fixtures; git diff --check; git diff --cached --check; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/README.md; docs/benchmarks/release_demo_human_review_queue_v1_2026-06-13.md; docs/specs/release_grade_musician_demo_bank_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 needed one stable handoff from source-family demo candidates to human listening review without letting unverified artifacts claim demo-ready quality.

## What Shipped

- Added a CI-safe release-demo human review queue generator, Justfile smoke target wired into audio-qa-ci, and benchmark/spec docs. The queue lists unverified candidates with source family, WAV, metrics, prompt refs, blocker context, priorities, and next review actions while enforcing human_verdict unverified, demo_readiness unverified, and quality_claim false.

## Notes

- None
