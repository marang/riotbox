# `RIOTBOX-1172` P016: Extract export action and receipt contract tests from oversized Core modules

- Ticket: `RIOTBOX-1172`
- Title: `P016: Extract export action and receipt contract tests from oversized Core modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1172/p016-extract-export-action-and-receipt-contract-tests-from-oversized`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1172-p016-export-contract-test-extraction`
- Linear branch: `feature/riotbox-1172-p016-extract-export-action-and-receipt-contract-tests-from`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1151 (https://github.com/marang/riotbox/pull/1151)`
- Merge commit: `9ab2f7824a8e3bafa523333daaebdcb40f685eaa`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; cargo test -p riotbox-core action::tests; cargo test -p riotbox-core session::export_types_tests; cargo test -p riotbox-core session::export_live_recording_contract_tests; cargo test -p riotbox-core; cargo test -p riotbox-app; just ci; GitHub rust-ci passed`
- Docs touched: `None`
- Follow-ups: `action.rs production code remains above 500 lines; split only when a semantic production boundary improves review cost without mixing behavior.`

## Why This Ticket Existed

RIOTBOX-1171 left export action and receipt contract tests inside oversized Core modules, increasing review cost for future P016 export slices.

## What Shipped

- Moved action tests into action_tests.rs and live-recording receipt contract tests into export_live_recording_contract_tests.rs as real semantic Rust test modules, keeping behavior unchanged and export_types_tests.rs under 500 lines.

## Notes

- Test-only cleanup. No runtime behavior, public contract, export action, receipt schema, observer, writer, or audio path changed.
