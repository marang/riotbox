# `RIOTBOX-1068` Add musician-facing export trigger and receipt feedback

- Ticket: `RIOTBOX-1068`
- Title: `Add musician-facing export trigger and receipt feedback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1068/add-musician-facing-export-trigger-and-receipt-feedback`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1068-export-trigger-feedback`
- Linear branch: `feature/riotbox-1068-add-musician-facing-export-trigger-and-receipt-feedback`
- Assignee: `Markus`
- Labels: None
- PR: `#1044 (https://github.com/marang/riotbox/pull/1044)`
- Merge commit: `c84f259ef264e7b6ff6b9fa71284084e3d4694f3`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-app jam_inspect_surfaces_latest_export_receipt_without_adding_perform_control`; `cargo test -p riotbox-app jam_inspect_surfaces_export_failure_feedback`; `cargo test -p riotbox-app shell_state_handles_help_refresh_and_action_keys`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1044`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `Split event_loop.rs semantically during a future event-loop/control slice; continue RIOTBOX-1069 export scope contract.`

## Why This Ticket Existed

P016 needed a musician-facing way to request the bounded product-mix export and read concise receipt or failure feedback without inventing a second export state.

## What Shipped

- Added `E` as the bounded Jam/TUI product-mix export trigger.
- Queued `export.product_mix` through the existing action path without export file I/O on the realtime path.
- Inspect mode now reports successful export receipt feedback with receipt id, status, mix+proof presence, and unsupported-scope note.
- Inspect mode now reports latest rejected/failed export feedback with action id and compact failure reason.
- Updated Action Lexicon and TUI spec with the trigger and no-second-export-truth boundary.

## Notes

- Full stem package export, DAW export, live recording export, host-audio soak, automatic arranger export, and Ghost export remain out of scope.
- event_loop.rs is now 496 lines and should be split semantically in a future event-loop/control slice.
