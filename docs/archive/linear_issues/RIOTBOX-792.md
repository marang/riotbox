# `RIOTBOX-792` Plan next P012 lane-output timing proof beyond MC-202 summary visibility

- Ticket: `RIOTBOX-792`
- Title: `Plan next P012 lane-output timing proof beyond MC-202 summary visibility`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-792/plan-next-p012-lane-output-timing-proof-beyond-mc-202-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-792-next-lane-output-timing-proof`
- Linear branch: `feature/riotbox-792-plan-next-p012-lane-output-timing-proof-beyond-mc-202`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#787 (https://github.com/marang/riotbox/pull/787)`
- Merge commit: `6a44abcb664ff28b953c649190e6a01dce67d455`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-792-feral-grid-pack-tests.log cargo test -p riotbox-audio --bin feral_grid_pack`; `scripts/run_compact.sh /tmp/riotbox-792-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-792-fmt.log cargo fmt --check`; `git diff --check`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed the next lane-output timing proof to measure the actual generated-support product surface, not keep adding summary-only checks.

## What Shipped

- Changed Feral-grid pack-level metrics.source_grid_output_drift to measure the complete generated-support mix while keeping TR-909 and W-30 lane-specific alignment metrics separate; updated Source Timing and Audio QA specs.

## Notes

- No audio rendering behavior changed; this corrected the QA measurement target for existing generated output.
