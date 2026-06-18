# `RIOTBOX-1286` Extract P023 professional-output JSON gates from Justfile into validators

- Ticket: `RIOTBOX-1286`
- Title: `Extract P023 professional-output JSON gates from Justfile into validators`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1286/extract-p023-professional-output-json-gates-from-justfile-into`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1286-professional-json-validators`
- Linear branch: `feature/riotbox-1286-extract-p023-professional-output-json-gates-from-justfile`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1261 (https://github.com/marang/riotbox/pull/1261)`
- Merge commit: `9c459697dca0235941c2731e1c96cd6d343f1e16`
- Deleted from Linear: `2026-06-18`
- Verification: `python3 -m py_compile scripts/validate_professional_output_listening_pack.py scripts/generate_mc202_producer_grade_closeout.py; just professional-output-listening-pack-smoke; just mc202-producer-grade-closeout-smoke; just pro-pressure-source-matrix-smoke; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The professional-output and MC-202 closeout smoke recipes had grown long inline jq contracts, making P023 release gates hard to review and easy to weaken accidentally.

## What Shipped

- Added a named professional-output listening-pack validator, added strict all-source-composed mode to the MC-202 closeout validator, shortened the Justfile smoke recipes, and documented that validator extraction preserves the human-verdict and no-primitive-template gates without making new musical quality claims.

## Notes

- None
