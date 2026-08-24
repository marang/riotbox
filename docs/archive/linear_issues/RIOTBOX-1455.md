# `RIOTBOX-1455` P023: Prove the exact live sparse-pressure journey through restart and recall

- Ticket: `RIOTBOX-1455`
- Title: `P023: Prove the exact live sparse-pressure journey through restart and recall`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1455/p023-prove-the-exact-live-sparse-pressure-journey-through-restart-and`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-22`
- Started: `2026-08-22`
- Finished: `2026-08-24`
- Branch: `feature/riotbox-1455-sparse-live-journey`
- Linear branch: `feature/riotbox-1455-p023-prove-the-exact-live-sparse-pressure-journey-through`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1438 (https://github.com/marang/riotbox/pull/1438)`
- Merge commit: `1ba1d75429382202e44ebfb1d9e0caa2e9bf9b70`
- Deleted from Linear: `2026-08-24`
- Verification: `cargo test -p riotbox-core w30_damage_policy`; `cargo test -p riotbox-audio mc202`; `cargo test -p riotbox-app w30_committed_bank_damage`; exact sparse v2 manifest validation; bounded structured human listening review; `just ci`; `GitHub rust-ci (PR #1438)`
- Docs touched: `README.md`, `docs/execution_roadmap.md`, `docs/jam_recipes.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1455_sparse_live_journey_acceptance_2026-08-24.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/replay_model_spec.md`
- Follow-ups: `None inside this slice; it does not authorize Holdout, universal-source, universal-hardness, bass, release, or zero-downtime restart claims.`

## Why This Ticket Existed

Prove that the already-qualified sparse-pressure held state and Transient Bite
contrast work as a complete musician path through capture, ordinary re-entry,
persistence, restart, recall, and exact live output rather than only as a
controlled render.

## What Shipped

- Made the existing W-30 damage action an explicit capture-scoped Apply/Bypass
  contract in the Session action log without adding another action or hidden
  app-local state.
- Proved capture, raw audition, promotion, held sparse pressure, Transient Bite,
  bypass/ordinary re-entry, save/restart, recall, and trigger through the exact
  RuntimeMix path with W-30, TR-909, and MC-202 contributor identity.
- Versioned MC-202 source-phrase transport sampling to absolute audio frames
  after the first run failed closed on callback-partition dependence.
- Recorded an artifact-bound structured human `keep` for both the held sparse
  transformation and its distinct Transient Bite state.

## Notes

- No new `ActionCommand`, `JamAppState` truth, source selector, fallback music,
  Holdout access, or commercial reference was introduced.
- The MC-202 correction changed transport determinism only; the frozen musical
  phrase, Sparse policy, Transient Bite algorithm, and thresholds were not tuned
  from source results.
