# `RIOTBOX-1502` Reconcile the professional Hook/Chop reverse-count gate with its QA contract

- Ticket: `RIOTBOX-1502`
- Title: `Reconcile the professional Hook/Chop reverse-count gate with its QA contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1502/reconcile-the-professional-hookchop-reverse-count-gate-with-its-qa`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-22`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1502-hook-chop-evidence`
- Linear branch: `feature/riotbox-1502-reconcile-the-professional-hookchop-reverse-count-gate-with`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1549 (https://github.com/marang/riotbox/pull/1549)`
- Merge commit: `f88db4e4e74ba86c32282c4b949a63839b08a95e`
- Deleted from Linear: `2026-09-30`
- Verification: `Full local just ci green; seven new Python fixture methods; 300 unchanged synthetic pattern comparisons; GitHub run 36767567637 green on Ubuntu and native Windows.`
- Docs touched: `docs/reviews/riotbox_1502_hook_chop_count_contract_2026-09-30.md`, `docs/specs/audio_qa/automated_qa.md`, `docs/research_decision_log.md`
- Follow-ups: `None required by this diagnostic-count slice.`

## Why This Ticket Existed

The professional diagnostic suite admitted one reverse gesture despite the strengthened two-gesture child contract.

## What Shipped

- Shared typed reverse-count validation now enforces two gestures across dense, tonal and suite evidence; malformed or missing counts fail closed, with source-free regression fixtures.

## Notes

- RBX-383 corrects provenance and metadata gates only. No historical hash-bound verdict rewrite, source or holdout access, render-policy change, human or musical pass. Windows CI was refreshed after the separate RIOTBOX-1506 readiness fix.
