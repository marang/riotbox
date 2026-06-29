# `RIOTBOX-1313` P023: Thread MC-202 role evidence into producer review queue

- Ticket: `RIOTBOX-1313`
- Title: `P023: Thread MC-202 role evidence into producer review queue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1313/p023-thread-mc-202-role-evidence-into-producer-review-queue`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1313-p023-thread-mc-202-role-evidence-into-producer-review-queue`
- Linear branch: `feature/riotbox-1313-p023-thread-mc-202-role-evidence-into-producer-review-queue`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1287 (https://github.com/marang/riotbox/pull/1287)`
- Merge commit: `592d0420859d4489ba26425ed9a598fb092aa7f1`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_mc202_real_source_listening_pack.py`; `just mc202-real-source-listening-pack-smoke`; `just mc202-producer-grade-closeout-smoke`; `just ci`; `GitHub Actions rust-ci passed for PR #1287`
- Docs touched: `docs/benchmarks/mc202_real_source_listening_pack_v1_2026-06-18.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

The MC-202 producer-grade closeout now distinguishes bass-pressure and answer/stab roles, but the human listening handoff still needed to show the same musical job to the reviewer. Without that, the software could be technically stricter while the reviewer-facing pack still looked like generic MC-202 evidence.

## What Shipped

- Added per-case mc202_role_evidence to the MC-202 real-source listening pack.
- Listening prompts and README tables now show whether each candidate should be judged as bass_pressure, pressure_answer, or hook_restraint_stab_answer.
- Role evidence is scoped as listening_review_target and keeps quality_proof false with human_verdict unverified.
- Validator mutation fixtures reject missing, stale, quality-claiming, dense-wrong, sparse-wrong, and tonal-wrong role evidence.
- Benchmark, Audio QA spec, and roadmap now document role-specific MC-202 review handoff.

## Notes

- This is a review-target handoff, not a producer-grade quality claim; structured listening still controls promotion.
