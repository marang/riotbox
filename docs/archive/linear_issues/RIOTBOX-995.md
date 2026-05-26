# `RIOTBOX-995` Make grid-use fixture downbeat evidence explicit

- Ticket: `RIOTBOX-995`
- Title: `Make grid-use fixture downbeat evidence explicit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-995/make-grid-use-fixture-downbeat-evidence-explicit`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-995-make-grid-use-fixture-downbeat-evidence-explicit`
- Linear branch: `feature/riotbox-995-make-grid-use-fixture-downbeat-evidence-explicit`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: `#987 (https://github.com/marang/riotbox/pull/987)`
- Merge commit: `131993ac5c6b30f06beab41a7baba6d44b24af70`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-995-grid-use-fixtures.log python3 scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-995-grid-use-just.log just source-timing-grid-use-contract-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-995-just-ci.log just ci`; `GitHub Actions Rust CI run 26449062176 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Make grid-use fixture downbeat score, margin, and alternate-phase evidence explicit so future ambiguity variants are reviewable from the case table.

## What Shipped

- Added explicit primary_downbeat_score, primary_downbeat_margin, and alternate_downbeat_phase_count fields to GridUseCase.
- Populated all existing grid-use contract cases with the previous generated values.
- Changed apply_timing_fields to copy downbeat evidence from each case instead of deriving it from downbeat_status.

## Notes

- Code touched: scripts/validate_source_timing_grid_use_contract_fixtures.py.
