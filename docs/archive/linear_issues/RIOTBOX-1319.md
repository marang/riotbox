# `RIOTBOX-1319` P023: Strengthen source-selection gates for weak and bad-timing outputs

- Ticket: `RIOTBOX-1319`
- Title: `P023: Strengthen source-selection gates for weak and bad-timing outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1319/p023-strengthen-source-selection-gates-for-weak-and-bad-timing-outputs`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1319-p023-strengthen-source-selection-gates-for-weak-and-bad`
- Linear branch: `feature/riotbox-1319-p023-strengthen-source-selection-gates-for-weak-and-bad`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1293 (https://github.com/marang/riotbox/pull/1293)`
- Merge commit: `4ce25b030ec1280aee8984f387067ac02be38ee7`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py scripts/generate_sound_quality_readiness_report.py scripts/validate_professional_output_suite_contract.py; git diff --check; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1319-edge-final; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1319-readiness-final; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1319-prof-suite-final; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `Pause requested by user after RIOTBOX-1319; do not start the next ticket until work resumes. plan/riotbox_improvement_readme.md remains an untracked working document and should be removed after its contents are incorporated into canonical docs.`

## Why This Ticket Existed

P023 readiness treated risky edge sources as weak/routed evidence without an explicit promotion gate. Bad-timing and pad/noise material must not silently become demo candidates before source-selection and timing review.

## What Shipped

- Added edge-source source-selection promotion gates, suite key metrics, readiness blocker, validator/mutation coverage, and roadmap/audio-QA documentation.

## Notes

- This is diagnostic/routing proof, not a musical-quality claim: edge cases remain quality_proof false and human_verdict unverified.
