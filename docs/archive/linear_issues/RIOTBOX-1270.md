# `RIOTBOX-1270` MC-202 structured listening review and demo-bank promotion gate

- Ticket: `RIOTBOX-1270`
- Title: `MC-202 structured listening review and demo-bank promotion gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1270/mc-202-structured-listening-review-and-demo-bank-promotion-gate`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1270-mc202-listening-review-promotion`
- Linear branch: `feature/riotbox-1270-mc-202-structured-listening-review-and-demo-bank-promotion`
- Assignee: `Markus`
- Labels: None
- PR: `#1245 (https://github.com/marang/riotbox/pull/1245)`
- Merge commit: `5a133e77`
- Deleted from Linear: `2026-06-15`
- Verification: `python3 -m py_compile scripts/mc202_source_composed_review_gate.py scripts/generate_professional_output_listening_pack.py scripts/promote_listening_review_to_demo_bank.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py; git diff --check; just professional-output-listening-pack-smoke; just demo-bank-promotion-fixtures; just professional-output-suite-smoke; just ci (/tmp/riotbox-1270-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/specs/release_grade_musician_demo_bank_spec.md`
- Follow-ups: `RIOTBOX-1264 parent remains open for actual structured human listening pass/fail and any production fixes from that verdict.`

## Why This Ticket Existed

MC-202 source-composed bass/answer candidates needed an explicit listening-review and demo-bank promotion boundary so automated diagnostics could not be mistaken for human-approved demo quality.

## What Shipped

- Added MC-202 source-composed gate metadata to professional listening packs, blocked primitive/template-only MC-202 review labels from demo-bank promotion, extended professional-output suite and promotion fixtures, and documented the boundary.

## Notes

- No structured human verdict was recorded; the gate intentionally keeps human_verdict unverified and quality_proof false until review.
