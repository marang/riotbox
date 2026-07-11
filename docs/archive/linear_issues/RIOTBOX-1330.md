# `RIOTBOX-1330` P023: Wire trusted Rust timing into live source ingest and confirmation

- Ticket: `RIOTBOX-1330`
- Title: `P023: Wire trusted Rust timing into live source ingest and confirmation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1330/p023-wire-trusted-rust-timing-into-live-source-ingest-and-confirmation`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-07-11`
- Finished: `2026-07-11`
- Branch: `feature/riotbox-1330-p023-wire-trusted-rust-timing-into-live-source-ingest-and`
- Linear branch: `feature/riotbox-1330-p023-wire-trusted-rust-timing-into-live-source-ingest-and`
- Assignee: `Markus`
- Labels: `Analysis`, `Audio`, `Core`, `timing`
- PR: `#1364 (https://github.com/marang/riotbox/pull/1364)`
- Merge commit: `7151a668247f61468f1b97e4be832556fcd90815`
- Deleted from Linear: `2026-07-11`
- Verification: `cargo test -p riotbox-app (546 passed); just check; full just ci; GitHub rust-ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1333`

## Why This Ticket Existed

Promote deterministic Rust source timing into live ingest and give all live timing consumers one Source Graph plus Session trust boundary.

## What Shipped

- Rust timing enrichment before graph/session persistence; readiness-gated TR-909, MC-202, W-30, Source Monitor, and transport timing; bounded explicit BPM confirmation through existing action/replay truth.

## Notes

- Contract enabler only; no audible quality or production-grade arbitrary-audio detector claim.
