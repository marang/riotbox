# `RIOTBOX-1541` Reject physically coupled Feral pack artifact destinations before rendering

- Ticket: `RIOTBOX-1541`
- Title: `Reject physically coupled Feral pack artifact destinations before rendering`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1541/reject-physically-coupled-feral-pack-artifact-destinations-before`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1541-feral-artifact-output-aliases`
- Linear branch: `feature/riotbox-1541-reject-physically-coupled-feral-pack-artifact-destinations`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1626 (https://github.com/marang/riotbox/pull/1626)`
- Merge commit: `e6b71be438817077d5f3b4aca151565e21d244bc`
- Deleted from Linear: `2026-10-01`
- Verification: `138 focused Debug/Release names/statuses match/pass; 141 CLI records and 85 hashes unchanged; source-free full CI zero; native run 36843955235 both required checks SUCCESS.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1542 independently confirms Before/After output-role corruption; all earlier guards stay intact.`

## Why This Ticket Existed

Feral pack accepted hardlinked TR-909/W-30 output roles and returned success with the same W-30 bytes under both names.

## What Shipped

- Reject all 171 physical pairs among the actual nineteen WAV/metrics/metadata destinations before analysis/rendering/publication; one private artifact iterator and retained physical identity backend.

## Notes

- Stable namespace only; no real sources/Holdout/commercial audio, DAW/playback/subagents or musical/release claim.
