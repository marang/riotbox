# `RIOTBOX-1174` P016: Add live-recording host-audio evidence refs

- Ticket: `RIOTBOX-1174`
- Title: `P016: Add live-recording host-audio evidence refs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1174/p016-add-live-recording-host-audio-evidence-refs`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1174-p016-live-recording-host-audio-evidence-refs`
- Linear branch: `feature/riotbox-1174-p016-add-live-recording-host-audio-evidence-refs`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1153 (https://github.com/marang/riotbox/pull/1153)`
- Merge commit: `0a5b5c7672500ca100f650edc334afaaf439e2e3`
- Deleted from Linear: `2026-06-03`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Live-recording receipts needed typed host-audio evidence refs in Session/Core before any future capture or writer can honestly claim readiness.

## What Shipped

- Added live_recording_host_audio_refs[] with host/device/duration/callback-gap/stream-error summaries, observer receipt projection from real action receipts, serde defaults for older receipts, and P016 spec updates that keep live recording reserved.

## Notes

- None
