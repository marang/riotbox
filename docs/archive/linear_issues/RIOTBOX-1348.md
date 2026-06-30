# `RIOTBOX-1348` P023: Strengthen tonal MC-202 mix-bus balance from producer routing

- Ticket: `RIOTBOX-1348`
- Title: `P023: Strengthen tonal MC-202 mix-bus balance from producer routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1348/p023-strengthen-tonal-mc-202-mix-bus-balance-from-producer-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1348-p023-strengthen-tonal-mc-202-mix-bus-balance-from-producer`
- Linear branch: `feature/riotbox-1348-p023-strengthen-tonal-mc-202-mix-bus-balance-from-producer`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1312 (https://github.com/marang/riotbox/pull/1312)`
- Merge commit: `575c1d54994c038e3d8584ec00ac974ba6f41b2a`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py scripts/mc202_producer_fix_routing.py; just professional-source-wav-pack-smoke; just professional-output-suite-smoke; just mc202-producer-grade-closeout-smoke; just demo-bank-promotion-fixtures; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/research_decision_log.md`
- Follow-ups: `Next P023 lane should record or route structured human listening verdicts; automated producer routing now leaves only human_listening.`

## Why This Ticket Existed

After RIOTBOX-1347, tonal hook_restraint cleared but tonal_rusharp_120 still routed to mix_bus because MC-202 support was below the 0.20 W-30 balance floor.

## What Shipped

- Raised tonal MC-202 support so tonal_mix_bus_mc202_to_w30_rms_ratio clears at 0.2082 while W-30 hook remains forward at 0.3344 with 0.1144 margin; professional suite now tracks this metric; closeout drops mix_bus and routes only to human_listening.

## Notes

- quality_proof remains false and human_verdict remains unverified; this is automated producer-routing evidence, not final musical approval.
