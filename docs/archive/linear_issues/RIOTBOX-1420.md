# `RIOTBOX-1420` Complete the project-wide audio numeric-value and magic-number audit

- Ticket: `RIOTBOX-1420`
- Title: `Complete the project-wide audio numeric-value and magic-number audit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1420/complete-the-project-wide-audio-numeric-value-and-magic-number-audit`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-21`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1420-audio-numeric-contract-audit`
- Linear branch: `feature/riotbox-1420-complete-the-project-wide-audio-numeric-value-and-magic`
- Assignee: `Markus`
- Labels: `Audio`, `Docs`, `Improvement`, `review-followup`
- PR: `#1539 (https://github.com/marang/riotbox/pull/1539)`
- Merge commit: `ff0a2434bdf22450b7dc7059b055a41b9801d544`
- Deleted from Linear: `2026-09-22`
- Verification: `Normal parallel just ci and GitHub rust-ci passed; ten numeric tests, nineteen dense-render tests, synthetic RuntimeMix/revalidation; sequential solo review.`
- Docs touched: `audio_numeric_values and bounded inventory; audio core/automated QA specs; RBX-379; branch review.`
- Follow-ups: `None`

## Why This Ticket Existed

Audit numeric ownership and prevent exact-QA producer/validator drift without changing sound or frozen source protocols.

## What Shipped

- Bounded nine-category passports; shared exact-mix predicates; unchanged shared suite thresholds; inclusive Alpha and supported Fill-boundary fixes; manifest/Graph format identity.

## Notes

- No DSP, source, holdout, DAW, device or human listening changes. RIOTBOX-1501 owns limiter calibration; RIOTBOX-1502 owns reverse-count contract reconciliation. Merge ff0a2434.
