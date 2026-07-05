# `RIOTBOX-1384` P023: Generate review packs from release-demo queue

- Ticket: `RIOTBOX-1384`
- Title: `P023: Generate review packs from release-demo queue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1384/p023-generate-review-packs-from-release-demo-queue`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1384-p023-generate-review-packs-from-release-demo-queue`
- Linear branch: `feature/riotbox-1384-p023-generate-review-packs-from-release-demo-queue`
- Assignee: `Markus`
- Labels: None
- PR: `#1348 (https://github.com/marang/riotbox/pull/1348)`
- Merge commit: `c874fb313d4fdaed8a1579c197268a9756cc0c96`
- Deleted from Linear: `2026-07-05`
- Verification: `just release-demo-listening-review-packs-fixtures via scripts/run_compact.sh; just listening-review-fixtures via scripts/run_compact.sh; just sound-quality-readiness-report-smoke via scripts/run_compact.sh; python3 -m py_compile scripts/generate_release_demo_listening_review_packs.py scripts/listening_review_workflow.py; git diff --check; GitHub rust-ci passed on PR #1348`
- Docs touched: `Justfile; docs/specs/audio_qa_workflow_spec.md; docs/specs/release_grade_musician_demo_bank_spec.md; scripts/generate_release_demo_listening_review_packs.py; scripts/validate_release_demo_listening_review_packs_fixtures.sh; scripts/listening_review_workflow.py`
- Follow-ups: `Use generated release-demo listening-review packs to record structured human verdicts for pad_noise, bad_timing, weak_source, sparse_drums, and any optional tonal follow-up candidate.`

## Why This Ticket Existed

P023 had a review queue and Markdown worklist, but no direct way to materialize each queue item as a local listening-review pack without manually copying candidate context.

## What Shipped

- Added a queue-to-pack generator that writes local unverified listening-review packs with preserved candidate context, artifact refs, blockers, verdict state, and questions; wired it into audio QA fixtures and allowed restore as a structured strongest element.

## Notes

- No rendered audio changed; generated packs remain review handoff artifacts with quality_claim false until human verdicts are recorded.
