# `RIOTBOX-957` Evaluate Source Timing grid counts from primary hypothesis

- Ticket: `RIOTBOX-957`
- Title: `Evaluate Source Timing grid counts from primary hypothesis`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-957/evaluate-source-timing-grid-counts-from-primary-hypothesis`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-957-evaluate-source-timing-grid-counts-from-primary-hypothesis`
- Linear branch: `feature/riotbox-957-evaluate-source-timing-grid-counts-from-primary-hypothesis`
- Assignee: `Markus`
- Labels: None
- PR: `#950 (https://github.com/marang/riotbox/pull/950)`
- Merge commit: `6f2c47daf8f5d983618758b8f289194179ba0a21`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-957-timing-tests.log cargo test -p riotbox-core source_graph::timing_tests -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-957-timing-tests-after-doc.log cargo test -p riotbox-core source_graph::timing_tests -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-957-just-ci-final.log just ci`; `GitHub Actions Rust CI run 26303355283 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

After Jam summary and grid-use policy started preferring selected primary-hypothesis grids, the Source Timing fixture evaluator still checked and reported only top-level TimingModel grid counts.

## What Shipped

- Updated fixture evaluation beat/bar/phrase count checks and serialized counts to use selected primary-hypothesis grids before top-level fallback.
- Added regression coverage for passing evaluator counts when top-level grids are empty but primary-hypothesis grids are present.
- Documented evaluator grid-count precedence in the Source Timing Intelligence spec.

## Notes

- Evaluator/spec slice only; no analyzer scoring, ActionCommand, queue, Session/replay, JamAppState, observer schema, realtime audio, or render behavior changed.
