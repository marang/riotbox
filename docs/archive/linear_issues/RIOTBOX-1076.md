# `RIOTBOX-1076` Make export recovery preflight artifact-set aware before wider scopes

- Ticket: `RIOTBOX-1076`
- Title: `Make export recovery preflight artifact-set aware before wider scopes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1076/make-export-recovery-preflight-artifact-set-aware-before-wider-scopes`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1076-artifact-set-recovery-preflight`
- Linear branch: `feature/riotbox-1076-make-export-recovery-preflight-artifact-set-aware-before`
- Assignee: `Markus`
- Labels: None
- PR: `#1051 (https://github.com/marang/riotbox/pull/1051)`
- Merge commit: `44261d4c01bd8cd852c27d94495fbf3fd318369e`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-app export_receipt_hydration_preflight -- --nocapture`; `cargo test -p riotbox-app recovery_surface -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1051`
- Docs touched: `docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1075, RIOTBOX-1077`

## Why This Ticket Existed

Recovery and hydration preflight must validate the typed artifact-set truth before wider export scopes can safely land.

## What Shipped

- Validated local-path artifact-set entries through export receipt preflight while preserving legacy product-mix artifact/proof return values.
- Kept URI artifact-set entries identity-only until a fetch/cache contract exists.
- Updated recovery artifact availability classification for artifact-set errors.
- Added preflight tests for existing local artifact-set entries, missing local artifact-set entries, and URI identity-only entries.

## Notes

- No stem writing, DAW export, live recording export, or URI fetch/cache behavior shipped.
