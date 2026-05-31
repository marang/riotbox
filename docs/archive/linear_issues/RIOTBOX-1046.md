# `RIOTBOX-1046` Add scene movement observer and audio correlation gate

- Ticket: `RIOTBOX-1046`
- Title: `Add scene movement observer and audio correlation gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1046/add-scene-movement-observer-and-audio-correlation-gate`
- Project: `P014 | Arrangement / Scene System`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-30`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1046-p014-scene-observer-audio-correlation`
- Linear branch: `feature/riotbox-1046-add-scene-movement-observer-and-audio-correlation-gate`
- Assignee: `Markus`
- Labels: None
- PR: `#1022 (https://github.com/marang/riotbox/pull/1022)`
- Merge commit: `a138b959e440cd5157f3a1c310db85f4a045b913`
- Deleted from Linear: `2026-05-31`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P014 needed a machine-checkable observer/audio proof that landed scene movement is correlated with Source Monitor anchor evidence and non-collapsed output metrics.

## What Shipped

- Added scene evidence to user-session observer snapshots for landed movement, Arrangement Scene readiness, source-locked movement permission, and Source Monitor anchor state.
- Extended observer_audio_correlate and audio-qa-ci with a P014 scene movement proof gate that rejects missing or collapsed evidence.

## Notes

- None
