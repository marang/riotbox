# `RIOTBOX-1505` Reject non-regular source WAVs without blocking Session restore

- Ticket: `RIOTBOX-1505`
- Title: `Reject non-regular source WAVs without blocking Session restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1505/reject-non-regular-source-wavs-without-blocking-session-restore`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1505-source-file-admission`
- Linear branch: `feature/riotbox-1505-reject-non-regular-source-wavs-without-blocking-session`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#1545 (https://github.com/marang/riotbox/pull/1545)`
- Merge commit: `f941affd02f2ea90d5106790868b7fc44be7071c`
- Deleted from Linear: `2026-09-30`
- Verification: `Both original public-interface tests reproduced the hang; focused Audio 22 and App 3 tests plus full just ci and GitHub rust-ci passed; solo Rust review and self-review with no retained findings.`
- Docs touched: `RBX-381; audio core spec; docs/reviews/riotbox_1505_source_file_admission_2026-09-30.md.`
- Follow-ups: `None`

## Why This Ticket Existed

Prevent invalid local source file types from stranding the P023 Session restore path.

## What Shipped

- Added shared Audio-owned opened-descriptor source admission, nonblocking Unix open, regular-source symlink compatibility and bounded FIFO regressions.

## Notes

- No DSP, ActionCommand, Session schema, human or DAW claim; regular-file I/O is not time/size bounded. RIOTBOX-1504 separately fixes Sidecar transport.
