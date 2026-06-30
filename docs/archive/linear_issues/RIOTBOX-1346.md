# `RIOTBOX-1346` P023: Strengthen dense destructive articulation from producer routing

- Ticket: `RIOTBOX-1346`
- Title: `P023: Strengthen dense destructive articulation from producer routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1346/p023-strengthen-dense-destructive-articulation-from-producer-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1346-p023-strengthen-dense-destructive-articulation-from-producer`
- Linear branch: `feature/riotbox-1346-p023-strengthen-dense-destructive-articulation-from-producer`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1310 (https://github.com/marang/riotbox/pull/1310)`
- Merge commit: `98d001a0416d37c0759baabc4d1cc082bbadca66`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile affected scripts; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1346-dense-pack-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just mc202-producer-grade-closeout-smoke; just audio-qa-ci; just ci; GitHub Rust CI/rust-ci`
- Docs touched: `docs/research_decision_log.md; docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `None`

## Why This Ticket Existed

After dense answer-bite passed, dense MC-202 still routed to destructive_articulation because the pressure-lift live gesture was just below the producer floor.

## What Shipped

- Strengthened dense pressure-lift articulation, added a named dense destructive articulation floor, gated it through pro-pressure matrix and professional-output suite, and removed dense destructive_articulation from closeout routing.

## Notes

- Dense now routes only to human_listening because structured human review remains unverified; automated gates do not claim human taste approval.
