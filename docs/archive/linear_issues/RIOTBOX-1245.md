# `RIOTBOX-1245` P023: Strengthen mix-bus clarity for source-backed professional output

- Ticket: `RIOTBOX-1245`
- Title: `P023: Strengthen mix-bus clarity for source-backed professional output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1245/p023-strengthen-mix-bus-clarity-for-source-backed-professional-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1245-p023-mix-bus-clarity`
- Linear branch: `feature/riotbox-1245-p023-strengthen-mix-bus-clarity-for-source-backed`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1220 (https://github.com/marang/riotbox/pull/1220)`
- Merge commit: `6f4e795cdb4e2e0d000254e495df4c1ce358fbc0`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py; just professional-output-suite-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Strengthened source-backed Feral mix-bus clarity so generated support no longer masks source-first renders while remaining audible in support renders.

## What Shipped

- Retuned source-first/generated-support mix policy; tightened Feral generated/source RMS gates; added Professional Output Suite feral_mix_balance aggregation with missing-field checks; documented the diagnostic suite contract.

## Notes

- Evidence remains diagnostic only: human_verdict unverified, not an automated musical pass.
