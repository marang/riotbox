# `RIOTBOX-1454` P023: Prove the exact live tonal-hook journey through restart and recall

- Ticket: `RIOTBOX-1454`
- Title: `P023: Prove the exact live tonal-hook journey through restart and recall`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1454/p023-prove-the-exact-live-tonal-hook-journey-through-restart-and`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-22`
- Started: `2026-08-22`
- Finished: `2026-08-22`
- Branch: `feature/riotbox-1454-p023-prove-the-exact-live-tonal-hook-journey-through-restart`
- Linear branch: `feature/riotbox-1454-p023-prove-the-exact-live-tonal-hook-journey-through-restart`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1436 (https://github.com/marang/riotbox/pull/1436)`
- Merge commit: `23c99afed90960b4817e45fb1e475d3e4f417f10`
- Deleted from Linear: `2026-08-22`
- Verification: `just controlled-source-live-matrix artifacts/audio_qa/local-controlled-source-live-matrix-riotbox-1454-final`; `just ci`; `GitHub rust-ci (PR #1436)`
- Docs touched: `README.md`, `docs/execution_roadmap.md`, `docs/jam_recipes.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1454_tonal_live_journey_v2_acceptance_2026-08-22.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: `None; the ticket does not reopen Pitch Dive design or authorize Holdout, universal-source, release, or zero-downtime restart claims.`

## Why This Ticket Existed

Prove that the already-qualified tonal W-30 behavior works as a complete musician path through capture, destructive contrast, ordinary re-entry, persistence, restart, recall, and exact live output.

## What Shipped

- Added typed TR-909 lead/support/stay-out intent so the tonal W-30 hook owns its held state while explicit performer overrides remain effective.
- Proved capture, held hook, Pitch Dive, ordinary re-entry, save/restart, recall, and trigger through the shared Session and exact RuntimeMix path.
- Passed the dense/tonal/sparse controlled Development matrix and recorded an artifact-bound human keep for the W-30-only tonal journey.

## Notes

- No new ActionCommand, JamAppState truth, DSP gesture, Holdout access, or commercial reference was introduced.
