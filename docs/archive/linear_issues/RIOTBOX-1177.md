# `RIOTBOX-1177` P016: Add live-recording readiness operator report skeleton

- Ticket: `RIOTBOX-1177`
- Title: `P016: Add live-recording readiness operator report skeleton`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1177/p016-add-live-recording-readiness-operator-report-skeleton`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1177-p016-live-recording-readiness-report`
- Linear branch: `feature/riotbox-1177-p016-add-live-recording-readiness-operator-report-skeleton`
- Assignee: `Markus`
- Labels: None
- PR: `#1156 (https://github.com/marang/riotbox/pull/1156)`
- Merge commit: `c1a000b51ced12686e6275dd606092d88282dedc`
- Deleted from Linear: `2026-06-04`
- Verification: `cargo test -p riotbox-app live_recording_report_cli`; `cargo test -p riotbox-app --test live_recording_report_smoke`; `cargo test -p riotbox-app`; `cargo test -p riotbox-core`; `git diff --check`; `just ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Live-recording host-audio receipt evidence now has typed readiness and observer projection, but operators still needed a read-only way to inspect a saved Session and understand whether the latest live-recording receipt is ready or blocked without enabling capture or WAV writing.

## What Shipped

- Added riotbox-app --live-recording-readiness-report --session <session.json> as a non-interactive, read-only CLI report.
- Reported missing live-recording receipts, typed host-audio readiness blockers, receipt identity, and host-audio refs from Session/Core truth.
- Added unit coverage plus a built-binary smoke for ready and blocked receipt evidence.
- Documented that the report writes no files, emits no observer events, launches no host, captures no audio, and does not make export.live_recording runnable.

## Notes

- PR #1156 merged after GitHub rust-ci passed.
