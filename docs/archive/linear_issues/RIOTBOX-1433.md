# `RIOTBOX-1433` P023: Audit audible algorithms against musician value and select bounded replacements

- Ticket: `RIOTBOX-1433`
- Title: `P023: Audit audible algorithms against musician value and select bounded replacements`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1433/p023-audit-audible-algorithms-against-musician-value-and-select`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-08-11`
- Started: `2026-08-11`
- Finished: `2026-08-14`
- Branch: `feature/riotbox-1433-algorithm-value-audit`
- Linear branch: `feature/riotbox-1433-p023-audit-audible-algorithms-against-musician-value-and`
- Assignee: `Markus`
- Labels: `review-followup`, `Spike`, `Analysis`, `Audio`, `Improvement`
- PR: `#1390 (https://github.com/marang/riotbox/pull/1390)`
- Merge commit: `933d6c8512f454779e856aa626cf2e8645d9b65c`
- Deleted from Linear: `Pending fresh token-authenticated cleanup; retained as Done`
- Verification: `git diff --check; just decision-search "RBX-284"; just ci; GitHub Rust CI; branch-level code review`
- Docs touched: `docs/README.md; docs/execution_roadmap.md; docs/phase_definition_of_done.md; docs/research_decision_log.md; docs/reviews/README.md; docs/reviews/riotbox_1433_audible_algorithm_value_audit_2026-08-12.md`
- Follow-ups: `RIOTBOX-1432`

## Why This Ticket Existed

Riotbox had accumulated several mathematically and operationally elaborate
audio algorithms without one compact comparison of their actual musician value.
The audit was needed to stop low-yield tuning and select one bounded product
algorithm whose replacement could audibly improve the Golden Path.

## What Shipped

- Recorded a concise retain, replace, and retire evidence map for product and
  near-product audible algorithm families.
- Selected product-owned W-30 hook-window choice as the sole immediate
  replacement target and retired further isolated F1–F4 force tuning from the
  active P023 priority lane.
- Froze RIOTBOX-1432 to the current one-bar baseline, exactly three registered
  Development sources, at most two explainable candidates, the existing
  capture/Session/replay/RuntimeMix spine, and one bounded human comparison
  after technical gates.
- Added Decision RBX-284 and aligned the P023 roadmap, phase DoD, and
  documentation map without creating a validator framework.

## Notes

- The audit recommendation used existing product and review evidence; no new
  human playback or Holdout access informed it.
- Normal repository CI exercised registered non-holdout regression fixtures,
  but those renders did not select or tune the recommendation.
- The archive helper could not authenticate to Linear, so this entry was
  created manually from the verified issue and PR records. The issue remains
  `Done` until fresh token-authenticated cleanup is available.
