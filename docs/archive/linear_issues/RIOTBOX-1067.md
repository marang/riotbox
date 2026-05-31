# `RIOTBOX-1067` Emit export product-mix observer lifecycle events

- Ticket: `RIOTBOX-1067`
- Title: `Emit export product-mix observer lifecycle events`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1067/emit-export-product-mix-observer-lifecycle-events`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1067-export-observer-lifecycle`
- Linear branch: `feature/riotbox-1067-emit-export-product-mix-observer-lifecycle-events`
- Assignee: `Markus`
- Labels: None
- PR: `#1043 (https://github.com/marang/riotbox/pull/1043)`
- Merge commit: `7daceab379232636e4cfaa77961d2d8a5c899246`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-app export_observer`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1043`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1068 for musician-facing export trigger and feedback`

## Why This Ticket Existed

The P016 export action boundary needed observer-visible lifecycle truth for product-mix export instead of forcing tools to infer export state from generic logs or receipts.

## What Shipped

- Added observer export lifecycle records for requested, started, completed, and failed product-mix export states.
- Completed export observer records now include receipt identity, role/boundary, artifact/proof paths, hashes, readiness status, machine-readable unsupported-scope flags, and musician-facing unsupported-scope labels.
- Failed export observer records include action id/status and failure reason without adding a second export truth.
- Documented the observer projection in the audio QA workflow spec.

## Notes

- The lifecycle projection is derived from ActionCommand, queue/action history, and export receipts; full stem, DAW, live recording, and host-audio soak remain out of scope.
