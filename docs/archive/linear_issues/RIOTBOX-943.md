# `RIOTBOX-943` Reject contradictory source-timing phrase evidence in probe JSON validator

- Ticket: `RIOTBOX-943`
- Title: `Reject contradictory source-timing phrase evidence in probe JSON validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-943/reject-contradictory-source-timing-phrase-evidence-in-probe-json`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-943-source-timing-probe-phrase-evidence-validator`
- Linear branch: `feature/riotbox-943-reject-contradictory-source-timing-phrase-evidence-in-probe`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#936 (https://github.com/marang/riotbox/pull/936)`
- Merge commit: `7b59990651ff733f247df2df85bfff7b3e00f2f4`
- Deleted from Linear: `2026-05-22`
- Verification: `just source-timing-probe-json-validator-fixtures; just ci; GitHub Actions Rust CI run 26297746070 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-944 continues by pinning cue/actionability in generated Feral-grid gates.`

## Why This Ticket Existed

Source-timing probe JSON exposed phrase status/count/bar-count evidence but validator accepted clear contradictions before downstream manifest and observer/audio QA.

## What Shipped

- Added stable, unavailable, and not_enough_material phrase-evidence consistency checks to the source-timing probe JSON validator plus focused mutated fixture checks.

## Notes

- QA contract hardening only; no runtime behavior changed.
