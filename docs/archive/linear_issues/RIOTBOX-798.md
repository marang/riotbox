# `RIOTBOX-798` Validate SourceGraphRef hash on session restore

- Ticket: `RIOTBOX-798`
- Title: `Validate SourceGraphRef hash on session restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-798/validate-sourcegraphref-hash-on-session-restore`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-798-source-graph-hash-restore`
- Linear branch: `feature/riotbox-798-validate-sourcegraphref-hash-on-session-restore`
- Assignee: `Markus`
- Labels: `timing`, `workflow`
- PR: `#793 (https://github.com/marang/riotbox/pull/793)`
- Merge commit: `cd8e28019f6b6ed2acdd16b71b1c41afb1c2bbb1`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-798-restore-tests.log cargo test -p riotbox-app restore_contracts -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-798-persistence-tests.log cargo test -p riotbox-app persistence_runtime_view -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-798-app-tests.log cargo test -p riotbox-app`; `scripts/run_compact.sh /tmp/riotbox-798-clippy-app.log cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-798-fmt.log cargo fmt --check`; `git diff --check`; `GitHub Actions Rust CI run 1918 passed on bde13a9c1c9b60d76548a78eb6dcd0b2521b0a26`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 restore safety requires sessions to reject a Source Graph whose content no longer matches SourceGraphRef.graph_hash, instead of silently restoring against different timing/source context.

## What Shipped

- Validated embedded, external, and explicit Source Graph loads against SourceGraphRef.graph_hash; refreshed graph hashes during save; added restore-contract tests for hash mismatches and persistence coverage proving refreshed hashes reload.

## Notes

- No Source Graph hash algorithm or multi-source restore behavior changed.
