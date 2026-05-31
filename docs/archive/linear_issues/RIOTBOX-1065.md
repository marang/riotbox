# `RIOTBOX-1065` Surface export receipts in inspect surfaces

- Ticket: `RIOTBOX-1065`
- Title: `Surface export receipts in inspect surfaces`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1065/surface-export-receipts-in-inspect-surfaces`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1065-export-receipts-inspect`
- Linear branch: `feature/riotbox-1065-surface-export-receipts-in-inspect-surfaces`
- Assignee: `Markus`
- Labels: None
- PR: `#1041 (https://github.com/marang/riotbox/pull/1041)`
- Merge commit: `3f0596802b1bb0adecb039abe711363662eef4bb`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-app export_readiness: pass`; `git diff --check: pass`; `just ci: pass`; `GitHub rust-ci on PR #1041: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

After export receipts exist, musicians need to inspect what was exported, where the proof lives, and what remains unsupported without a second export truth.

## What Shipped

- Surfaced the latest Session export receipt in Jam inspect Material flow.
- Included export role, boundary, compact receipt id, readiness status, artifact/proof path presence, and unsupported scope labels within the existing panel height.
- Kept the previous export-readiness hint when no receipt exists.
- Kept Perform free of export-control claims.

## Notes

- None
