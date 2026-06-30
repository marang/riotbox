# `RIOTBOX-1349` P023: Turn MC-202 human-listening closeout into concrete review queue

- Ticket: `RIOTBOX-1349`
- Title: `P023: Turn MC-202 human-listening closeout into concrete review queue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1349/p023-turn-mc-202-human-listening-closeout-into-concrete-review-queue`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1349-p023-turn-mc-202-human-listening-closeout-into-concrete`
- Linear branch: `feature/riotbox-1349-p023-turn-mc-202-human-listening-closeout-into-concrete`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1313 (https://github.com/marang/riotbox/pull/1313)`
- Merge commit: `906cfa187fdd4e451f4c655059e03e327ccf7ff5`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_mc202_producer_grade_closeout.py; just professional-output-listening-pack-smoke; just mc202-producer-grade-closeout-smoke; just demo-bank-promotion-fixtures; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/research_decision_log.md`
- Follow-ups: `Next P023 slice should consume structured listening verdicts or make the review queue easier for a human listener to execute without weakening quality_proof boundaries.`

## Why This Ticket Existed

After RIOTBOX-1348, automated producer routing only left human_listening; that needed to become concrete auditable review work rather than a vague closeout category.

## What Shipped

- MC-202 producer-grade closeout now emits a structured_listening_review_queue with three entries for dense, tonal, and sparse candidates; each entry includes exact WAV, review JSON, prompt, metrics, hashes, role evidence, route category, and quality-proof boundaries; validation mutation-tests missing/stale/quality-claim cases.

## Notes

- quality_proof remains false and automated_musical_approval remains false; this queue is review work, not musical approval.
