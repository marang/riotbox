# `RIOTBOX-69` Ticket Archive

- Ticket: `RIOTBOX-69`
- Title: `Make W-30 preview mode explicit instead of deriving it from action-log history`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-69/make-w-30-preview-mode-explicit-instead-of-deriving-it-from-action-log`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-69-w30-preview-mode-state`
- Linear branch: `feature/riotbox-69-make-w-30-preview-mode-explicit-instead-of-deriving-it-from`
- Assignee: `Markus`
- Labels: `None`
- PR: `#63`
- Merge commit: `06eb02d`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`, `self-review`, `GitHub Actions Rust CI`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-67`, `RIOTBOX-71`

## Why This Ticket Existed

The periodic review in `RIOTBOX-68` found that the W-30 preview seam still reconstructed current preview intent from `action_log` history instead of one committed lane-state field. That left replay and restore semantics too dependent on older committed actions and made later W-30 controls harder to reason about cleanly.

## What Shipped

- Added explicit persisted W-30 preview intent to `W30LaneState` as `preview_mode`.
- Made committed W-30 preview-facing actions update that lane-state field directly.
- Stopped the live preview builder from reconstructing preview mode from `action_log` ordering after load.
- Added a one-time legacy backfill path so older sessions load into explicit preview intent once and then stay on the committed-state model.
- Added regression coverage for both legacy backfill and explicit-state-overrides-history behavior.
- Updated the session contract and decision log so the preview-mode rule is now documented instead of implicit.

## Notes

- This slice intentionally changed state authority, not user-facing W-30 controls. Later W-30 actions should keep updating preview intent through committed lane state rather than adding new history-derived shortcuts.
- The explicit preview-mode field keeps the W-30 seam aligned with the broader repo rule that replay- and restore-relevant behavior must live in deterministic model state.
