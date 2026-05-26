# `RIOTBOX-989` Add end-to-end source transport map capture workflow QA proof

- Ticket: `RIOTBOX-989`
- Title: `Add end-to-end source transport map capture workflow QA proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-989/add-end-to-end-source-transport-map-capture-workflow-qa-proof`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-989-add-end-to-end-source-transport-map-capture-workflow-qa-proof`
- Linear branch: `feature/riotbox-989-add-end-to-end-source-transport-map-capture-workflow-qa`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#981 (https://github.com/marang/riotbox/pull/981)`
- Merge commit: `e694a016b2f8aec3640e78190097f04e29a24c5b`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-989-rebased-user-session.log cargo test -p riotbox-app --bin user_session_observer_probe -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-989-rebased-probe.log just source-transport-map-capture-probe`; `scripts/run_compact.sh /tmp/riotbox-989-rebased-ci.log just ci`
- Docs touched: `docs/jam_recipes.md`, `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Close the Source Transport / Map / Capture parent workflow with one coherent musician-path QA proof that includes observer and output evidence.

## What Shipped

- Added source-transport-map-capture headless observer probe.
- Added just source-transport-map-capture-probe and wired it into audio QA CI.
- Exercised listen-first confirmation, Source Map seek, source-window-backed capture, raw audition, promote, W-30 trigger, observer evidence, and output comparison.

## Notes

- Output proof after rebase: W-30 source-vs-fallback comparison passed through just source-transport-map-capture-probe.
