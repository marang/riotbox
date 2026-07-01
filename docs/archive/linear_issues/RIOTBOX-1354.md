# `RIOTBOX-1354` P023: Strengthen mix-bus impact without source masking

- Ticket: `RIOTBOX-1354`
- Title: `P023: Strengthen mix-bus impact without source masking`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1354/p023-strengthen-mix-bus-impact-without-source-masking`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1354-p023-strengthen-mix-bus-impact-without-source-masking`
- Linear branch: `feature/riotbox-1354-p023-strengthen-mix-bus-impact-without-source-masking`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1318 (https://github.com/marang/riotbox/pull/1318)`
- Merge commit: `7838252214fe8fbbf9910560f9d00cc6ac29f726`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py scripts/validate_automated_musical_fitness.py`; `cargo fmt --check`; `cargo test -p riotbox-audio --bin feral_grid_pack`; `git diff --check`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1354-professional-output-suite-third`; `just sound-quality-readiness-report-smoke`; `just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1354-weak-routing`; `just automated-musical-fitness-fixtures`; `just representative-source-showcase-musical-quality-fixtures`; `just source-family-release-demo-coverage-fixtures artifacts/audio_qa/local-riotbox-1354-source-family-coverage`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1354-professional-source-wav-pack`; `just syncopated-source-showcase-smoke`; `just audio-qa-ci`; `just ci`; `GitHub Actions rust-ci passed on PR #1318`
- Docs touched: `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 weak-output routing still named mix_bus as a recurring production-fix category and as the primary fix for automated_source_masked. The ticket existed to make generated support more useful without letting source-first output hide the transformed W-30/source character.

## What Shipped

- Added adaptive generated-support shaping for drop-contour Feral-grid renders so low-support cases are boosted to a useful floor while already-strong cases avoid source-masking.
- Raised the professional-output mix-bus contract: minimum support generated/source ratio is now 0.145, source-first masking headroom is 0.04, and the 0.08 source-first masking ceiling remains enforced.
- Mirrored the support floor through Rust pack validation, manifests, text reports, manifest assertions, professional-output validators, automated-fitness fixtures, roadmap, audio QA spec, and decision log.
- Improved TR-909 rendered-pressure failure output with all-lane mix details so future mix failures explain the specific blocked proof.

## Notes

- Human verdict remains unverified; this is stronger diagnostic/output-path proof, not a release-grade musical pass claim.
