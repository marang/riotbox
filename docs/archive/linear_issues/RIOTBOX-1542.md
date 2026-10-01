# `RIOTBOX-1542` Reject coupled Before/After pack output roles before the early excerpt write

- Ticket: `RIOTBOX-1542`
- Title: `Reject coupled Before/After pack output roles before the early excerpt write`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1542/reject-coupled-beforeafter-pack-output-roles-before-the-early-excerpt`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1542-before-after-output-aliases`
- Linear branch: `feature/riotbox-1542-reject-coupled-beforeafter-pack-output-roles-before-the`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1628 (https://github.com/marang/riotbox/pull/1628)`
- Merge commit: `1d0e8444b0e79d7e99e812eb2b1d141d012f8a9b`
- Deleted from Linear: `2026-10-01`
- Verification: `143 exact matching passing Debug/Release names/statuses; 141 CLI records and 85 hashes unchanged; source-free full CI zero and scans clear; native run 36846054936 both required checks SUCCESS.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1543 is a mechanical Feral orchestration/report ownership migration, not more audio-policy tuning.`

## Why This Ticket Existed

Before/After accepted hardlinked W-30/TR-909 destinations and returned success with the same last-written TR-909 bytes under both names.

## What Shipped

- Reject all 91 physical output pairs before the early excerpt write, with one actual fourteen-file iterator; necessary shared mutual-output preflight now serves four existing QA consumers without duplicated identity logic.

## Notes

- All 37 old safety bodies retained; original three collision loops preserve inputs/outputs. Stable namespace only, no real sources/Holdout/commercial audio/DAW/playback/subagents or qualification claims.
