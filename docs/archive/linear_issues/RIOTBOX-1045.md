# `RIOTBOX-1045` Add P014 scene timing trust safety matrix

- Ticket: `RIOTBOX-1045`
- Title: `Add P014 scene timing trust safety matrix`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1045/add-p014-scene-timing-trust-safety-matrix`
- Project: `P014 | Arrangement / Scene System`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-30`
- Started: `2026-05-30`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1045-p014-scene-timing-trust`
- Linear branch: `feature/riotbox-1045-add-p014-scene-timing-trust-safety-matrix`
- Assignee: `Markus`
- Labels: None
- PR: `#1021 (https://github.com/marang/riotbox/pull/1021)`
- Merge commit: `f81187a0d4542be50f85d385aa44900cf239c1b7`
- Deleted from Linear: `2026-05-31`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P014 needed an explicit timing-trust matrix so scene movement could not silently treat fallback or manual timing as source-locked truth.

## What Shipped

- Added P014 Arrangement Scene timing trust coverage across locked, user-confirmed, manual-confirm, fallback, disabled, missing BPM, and missing Source Graph states.
- Kept Source Monitor scene anchors limited to trusted timing states and documented the bounded source-reposition contract.

## Notes

- None
