# `RIOTBOX-1342` P023: Route MC-202 listening weaknesses into producer-grade fix candidates

- Ticket: `RIOTBOX-1342`
- Title: `P023: Route MC-202 listening weaknesses into producer-grade fix candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1342/p023-route-mc-202-listening-weaknesses-into-producer-grade-fix`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1342-p023-route-mc-202-listening-weaknesses-into-producer-grade`
- Linear branch: `feature/riotbox-1342-p023-route-mc-202-listening-weaknesses-into-producer-grade`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1306 (https://github.com/marang/riotbox/pull/1306)`
- Merge commit: `be34d904`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_mc202_producer_grade_closeout.py scripts/mc202_producer_fix_routing.py: pass; just mc202-producer-grade-closeout-smoke: pass; cargo fmt --check: pass; cargo test -p riotbox-audio: pass; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings: pass; just ci: pass; GitHub rust-ci: pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1343: consume mc202_producer_fix_candidates when recording structured MC-202 human verdicts`

## Why This Ticket Existed

P023 needed weak or unverified MC-202 producer-grade review candidates to become structured fix work instead of prose-only notes or accidental quality claims.

## What Shipped

- The MC-202 producer-grade closeout now emits per-candidate fix routes plus aggregate producer fix candidates with exact WAV refs, software next steps, musician payoff, and quality-proof boundaries.

## Notes

- Producer fix candidates remain work-selection evidence only: quality_proof=false and automated_musical_approval=false until structured listening review accepts the audio.
