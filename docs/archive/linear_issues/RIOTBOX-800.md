# `RIOTBOX-800` Add explicit source graph path hash restore regression

- Ticket: `RIOTBOX-800`
- Title: `Add explicit source graph path hash restore regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-800/add-explicit-source-graph-path-hash-restore-regression`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-800-explicit-graph-hash-restore`
- Linear branch: `feature/riotbox-800-add-explicit-source-graph-path-hash-restore-regression`
- Assignee: `Markus`
- Labels: `timing`, `workflow`
- PR: `#794 (https://github.com/marang/riotbox/pull/794)`
- Merge commit: `2771d0e1d5757c56664ddcb5f2343c01c6c1703a`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-800-restore-tests.log cargo test -p riotbox-app restore_contracts -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-800-clippy-app.log cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-800-fmt.log cargo fmt --check`; `git diff --check`; `GitHub Actions Rust CI run 1921 passed on 9bd79b00e55593031a073c8c7e0beccff27e335d`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-798 validated explicit Source Graph paths in code but lacked a direct restore regression for the explicit override path.

## What Shipped

- Added a focused restore-contract test proving an explicit Source Graph path whose content hash differs from the session SourceGraphRef.graph_hash is rejected with InvalidSession.

## Notes

- Behavior was already intended by RIOTBOX-798; this slice adds direct coverage only.
