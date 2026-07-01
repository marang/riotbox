# `RIOTBOX-1356` P023: Strengthen TR-909 drum pressure from weak-output routing

- Ticket: `RIOTBOX-1356`
- Title: `P023: Strengthen TR-909 drum pressure from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1356/p023-strengthen-tr-909-drum-pressure-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1356-p023-strengthen-tr-909-drum-pressure-from-weak-output`
- Linear branch: `feature/riotbox-1356-p023-strengthen-tr-909-drum-pressure-from-weak-output`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1320 (https://github.com/marang/riotbox/pull/1320)`
- Merge commit: `10e96fec`
- Deleted from Linear: `2026-07-01`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio --bin feral_grid_pack tr909_rendered_drum_pressure -- --nocapture; cargo test -p riotbox-audio --bin feral_grid_pack source_aware_tr909_profile_changes_for_same_bpm_sources -- --nocapture; just syncopated-source-showcase-smoke; just professional-output-suite-smoke; just sound-quality-readiness-report-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Routed drum_pressure fixes needed rendered-output proof that TR-909 pressure survives as physical support instead of decorative or buried drum presence.

## What Shipped

- Tightened rendered TR-909 proof to require 0.05 support contribution for every source profile, kept drop-drive/break-lift at 0.0030 low-band RMS, raised steady-pulse to 0.0017, strengthened break-lift and steady-pulse kick body, and preserved source-first/support masking ceilings.

## Notes

- Evidence remains diagnostic with human_verdict unverified and quality_proof false.
