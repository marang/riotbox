# `RIOTBOX-942` Reject contradictory source-timing phrase evidence in listening manifest validator

- Ticket: `RIOTBOX-942`
- Title: `Reject contradictory source-timing phrase evidence in listening manifest validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-942/reject-contradictory-source-timing-phrase-evidence-in-listening`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-942-listening-manifest-phrase-evidence-validator`
- Linear branch: `feature/riotbox-942-reject-contradictory-source-timing-phrase-evidence-in`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#935 (https://github.com/marang/riotbox/pull/935)`
- Merge commit: `432422109f33c01f19da03d896ca69d36805e8e0`
- Deleted from Linear: `2026-05-22`
- Verification: `just listening-manifest-validator-fixtures; just ci; GitHub Actions Rust CI run 26297328314 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-943 continues by applying the same contradiction checks to source-timing probe JSON.`

## Why This Ticket Existed

Listening manifests exposed phrase status/count/bar-count evidence but validator accepted clear contradictions before downstream observer/audio QA.

## What Shipped

- Added stable, unavailable, and not_enough_material phrase-evidence consistency checks to the listening manifest validator plus focused mutated fixture checks.

## Notes

- QA contract hardening only; no runtime behavior changed.
