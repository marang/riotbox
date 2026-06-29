# `RIOTBOX-1314` P023: Require MC-202 role evidence for demo-bank promotion

- Ticket: `RIOTBOX-1314`
- Title: `P023: Require MC-202 role evidence for demo-bank promotion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1314/p023-require-mc-202-role-evidence-for-demo-bank-promotion`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1314-p023-require-mc-202-role-evidence-for-demo-bank-promotion`
- Linear branch: `feature/riotbox-1314-p023-require-mc-202-role-evidence-for-demo-bank-promotion`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1288 (https://github.com/marang/riotbox/pull/1288)`
- Merge commit: `d5e9dea93bcb93ec2c8f2661dc6b360b91f20577`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/mc202_source_composed_review_gate.py scripts/generate_professional_output_listening_pack.py scripts/promote_listening_review_to_demo_bank.py scripts/validate_release_grade_demo_bank.py; just release-grade-demo-bank-fixtures; just demo-bank-promotion-fixtures; just professional-output-listening-pack-smoke; just mc202-producer-grade-closeout-smoke; just ci; GitHub rust-ci pass on PR #1288`
- Docs touched: `docs/execution_roadmap.md; docs/specs/release_grade_musician_demo_bank_spec.md; docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

Demo-bank promotion must not strip the MC-202 musical role that human reviewers actually judged.

## What Shipped

- Required and preserved source-family-matched mc202_role_evidence through professional listening labels, promotion, and release-grade demo-bank validation.

## Notes

- Also documented targeted audio-QA selection: run seam-specific checks first and reserve broad audio smokes/just ci for release, promotion, shared validator, render-policy, or cross-cutting risk.
