# `RIOTBOX-940` Reject contradictory source-timing phrase evidence in observer/audio summary validator

- Ticket: `RIOTBOX-940`
- Title: `Reject contradictory source-timing phrase evidence in observer/audio summary validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-940/reject-contradictory-source-timing-phrase-evidence-in-observeraudio`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-940-source-timing-phrase-evidence-validator`
- Linear branch: `feature/riotbox-940-reject-contradictory-source-timing-phrase-evidence-in`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#933 (https://github.com/marang/riotbox/pull/933)`
- Merge commit: `36f72a9150a5d32d4fd93cad53f608a4a8dd5405`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-summary-validator-fixtures; just source-timing-grid-use-contract-fixtures; just ci; GitHub Actions Rust CI run 26296410093 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-941 continues by tightening user-session observer phrase-lock consistency.`

## Why This Ticket Existed

Observer/audio summaries exposed phrase evidence but validator accepted clear contradictions between phrase status and phrase counters.

## What Shipped

- Added semantic phrase-evidence checks for stable, unavailable, and not_enough_material source_timing summary states; added mutated fixture checks; updated grid-use contract generated fixtures to emit matching phrase evidence.

## Notes

- QA hardening only; no runtime behavior changed. First local just ci caught a generated grid-use contract fixture with stable phrase_status and zero phrase evidence; fixed in the same slice.
