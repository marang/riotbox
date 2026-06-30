# `RIOTBOX-1343` P023: Record structured MC-202 human verdicts for demo-bank promotion

- Ticket: `RIOTBOX-1343`
- Title: `P023: Record structured MC-202 human verdicts for demo-bank promotion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1343/p023-record-structured-mc-202-human-verdicts-for-demo-bank-promotion`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1343-p023-record-structured-mc-202-human-verdicts-for-demo-bank`
- Linear branch: `feature/riotbox-1343-p023-record-structured-mc-202-human-verdicts-for-demo-bank`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1307 (https://github.com/marang/riotbox/pull/1307)`
- Merge commit: `175aa93e`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/listening_review_workflow.py scripts/promote_listening_review_to_demo_bank.py scripts/validate_release_grade_demo_bank.py: pass; just listening-review-fixtures: pass; just demo-bank-promotion-fixtures: pass; python3 scripts/validate_release_grade_demo_bank.py scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json: pass; just professional-output-listening-verdict-import-fixtures: pass; just audio-qa-ci: pass; just ci: pass; GitHub rust-ci: pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/release_grade_musician_demo_bank_spec.md; docs/research_decision_log.md`
- Follow-ups: `Continue P023 with the next producer-fix candidate from mc202_producer_fix_candidates, starting with bass_movement if still top-ranked.`

## Why This Ticket Existed

P023 needed structured MC-202 human verdict promotion to consume RIOTBOX-1342 producer fix routing by exact case and WAV hash instead of manual free-form categories.

## What Shipped

- Listening reviews now record demo-readiness consequence; demo-bank promotion can consume MC-202 closeout fix candidates, derive weak/fail fix categories, reject stale closeout hashes and manual mismatches, and preserve no-quality-proof boundaries.

## Notes

- Human pass still carries no fix categories; weak/fail entries remain not_demo_ready and keep concrete producer fix categories only after exact artifact match.
