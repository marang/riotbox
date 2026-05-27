# `RIOTBOX-1011` P012: Wire Recipe 15 strict missing-fixture guard into audio QA CI

- Ticket: `RIOTBOX-1011`
- Title: `P012: Wire Recipe 15 strict missing-fixture guard into audio QA CI`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1011/p012-wire-recipe-15-strict-missing-fixture-guard-into-audio-qa-ci`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1011-recipe15-strict-fixture-audio-qa`
- Linear branch: `feature/riotbox-1011-p012-wire-recipe-15-strict-missing-fixture-guard-into-audio`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `workflow`
- PR: `#995 (https://github.com/marang/riotbox/pull/995)`
- Merge commit: `680dd6e58fa3103b2dc50e41f81be0cd77056471`
- Deleted from Linear: `2026-05-27`
- Verification: `just recipe15-strict-missing-fixture-fixture`; `scripts/run_compact.sh /tmp/riotbox-1011-audio-qa-ci.log just audio-qa-ci`; `GitHub Actions Rust CI run 26494966210 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 required the phase-level audio QA gate to fail when required Recipe 15 source fixtures are missing; the strict fixture proof existed but was not part of audio-qa-ci.

## What Shipped

- Wired recipe15-strict-missing-fixture-fixture into just audio-qa-ci.
- Kept default CI independent of local real-source WAVs by using the temporary missing-path fixture proof.

## Notes

- None
