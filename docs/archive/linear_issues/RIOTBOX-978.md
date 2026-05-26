# `RIOTBOX-978` Add source timing confirmation observer probe

- Ticket: `RIOTBOX-978`
- Title: `Add source timing confirmation observer probe`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-978/add-source-timing-confirmation-observer-probe`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-978-add-source-timing-confirmation-observer-probe`
- Linear branch: `feature/riotbox-978-add-source-timing-confirmation-observer-probe`
- Assignee: `Markus`
- Labels: `Feature`, `timing`
- PR: `#970 (https://github.com/marang/riotbox/pull/970)`
- Merge commit: `481db0624976af2c6b925b975233df53e2a65e52`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-978-rebased-observer-test.log cargo test -p riotbox-app writes_source_timing_confirmation_observer_stream -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-978-rebased-probe.log just source-timing-confirmation-probe`; `scripts/run_compact.sh /tmp/riotbox-978-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Give the source timing confirmation path a repeatable observer/probe proof instead of relying only on unit tests and render assertions.

## What Shipped

- Added source-timing-confirmation to the headless user-session observer probe.
- Pressed the real C shell outcome and asserted immediate source_timing.confirm_grid commit evidence.
- Added validate_source_timing_confirmation_probe.sh, just source-timing-confirmation-probe, and included the probe in audio QA CI.

## Notes

- Rebased onto current main after RIOTBOX-977 merged, retargeted PR #970 to main, then reran local focused tests, probe, and just ci.
