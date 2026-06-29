# `RIOTBOX-1309` P023: Strengthen TR-909 drum pressure in rendered output path

- Ticket: `RIOTBOX-1309`
- Title: `P023: Strengthen TR-909 drum pressure in rendered output path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1309/p023-strengthen-tr-909-drum-pressure-in-rendered-output-path`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1309-p023-strengthen-tr-909-drum-pressure-in-rendered-output-path`
- Linear branch: `feature/riotbox-1309-p023-strengthen-tr-909-drum-pressure-in-rendered-output-path`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1283 (https://github.com/marang/riotbox/pull/1283)`
- Merge commit: `7015aabd`
- Deleted from Linear: `2026-06-29`
- Verification: `GitHub rust-ci; python3 -m py_compile scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py scripts/generate_sound_quality_readiness_report.py; cargo test -p riotbox-audio --bin feral_grid_pack tr909_rendered_drum_pressure -- --nocapture; cargo test -p riotbox-audio --bin feral_grid_pack -- --nocapture; cargo clippy --all-targets --all-features -- -D warnings; just listening-manifest-validate-generated-packs; just beat03-auto-feral-grid-proof`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `Continue P023 queued follow-ups: RIOTBOX-1310 fixture thresholds and RIOTBOX-1311 source/timing UI cue. Known current blocker: professional-source-wav-pack-smoke fails independently on tonal_rusharp_120 and blocks professional-output-suite-smoke.`

## Why This Ticket Existed

P023 weak-output routing flagged drum_pressure: Riotbox needed proof that source-derived TR-909 pressure survives the rendered support mix without masking source-first output.

## What Shipped

- Rebalanced generated-support mix toward TR-909 body while preserving generated/source ceilings, added tr909_rendered_drum_pressure proof to Feral report/manifest/readiness/professional-suite summaries, added Rust regressions for accepted vs buried rendered pressure, and documented the boundary in the roadmap.

## Notes

- Beat03 after rebalance reported tr909_rendered_drum_pressure.applied=true, support contribution 0.05870868, source-first generated/source 0.016811088, support generated/source 0.18754254. Evidence remains diagnostic with human_verdict unverified.
