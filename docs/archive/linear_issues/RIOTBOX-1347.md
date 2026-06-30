# `RIOTBOX-1347` P023: Strengthen tonal MC-202 hook restraint from producer routing

- Ticket: `RIOTBOX-1347`
- Title: `P023: Strengthen tonal MC-202 hook restraint from producer routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1347/p023-strengthen-tonal-mc-202-hook-restraint-from-producer-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1347-p023-strengthen-tonal-mc-202-hook-restraint-from-producer`
- Linear branch: `feature/riotbox-1347-p023-strengthen-tonal-mc-202-hook-restraint-from-producer`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1311 (https://github.com/marang/riotbox/pull/1311)`
- Merge commit: `33e457fac25d2ee378a9159f6fb5b44a9d01e1f2`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/mc202_producer_fix_routing.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py; just professional-source-wav-pack-smoke; just professional-output-suite-smoke; just mc202-producer-grade-closeout-smoke; just demo-bank-promotion-fixtures; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/research_decision_log.md`
- Follow-ups: `Next P023 slice: tonal mix_bus remains routed because mc202_to_w30_rms_ratio is below the 0.20 floor.`

## Why This Ticket Existed

Tonal MC-202 hook_restraint still routed as a broad tonal bucket, hiding whether hook restraint or mix balance was the actual next producer problem.

## What Shipped

- Tonal hook_restraint now routes only when pressure_low_band_lift_ratio falls below 2.20; tonal pressure support clears at 2.2407 while W-30 hook stays forward; closeout now routes tonal to mix_bus instead of hook_restraint.

## Notes

- quality_proof remains false and human_verdict remains unverified; this is automated producer-routing evidence, not final listening approval.
