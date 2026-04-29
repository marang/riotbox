# P010 Ghost Watch / Assist Exit Review

Date: 2026-04-29
Scope: `docs/specs/ghost_api_spec.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, and the current Ghost Watch / Assist implementation.

## Verdict

P010 is exit-ready for the bounded MVP meaning of Ghost Watch / Assist.

The implemented path is intentionally narrow:

- Ghost can create a source-backed Jam-state suggestion today.
- The user must explicitly accept or reject the suggestion.
- Accepted suggestions become normal `ActorType::Ghost` action queue entries.
- Watch remains read-only.
- Assist acceptance respects pending, phrase, destructive-scene, and lock safety gates.
- Accepted Ghost actions are logged through the normal action log and commit metadata path.

This does not mean Riotbox is ready for autonomous `perform` mode. The Ghost API still correctly lists `perform` escalation as a future follow-up.

## Evidence

- `watch` and `assist` are the MVP-supported modes; `perform` remains disabled until replay and action safety are proven: `docs/specs/ghost_api_spec.md:68`.
- Ghost proposal fields are documented as musician-facing and replay/audit-friendly, with explicit acceptability and blocker rules: `docs/specs/ghost_api_spec.md:234`.
- The approved flow uses normal `ActionDraft`s, the normal action queue, and quantized commit boundaries: `docs/specs/ghost_api_spec.md:263`.
- Budget enforcement is documented for pending, phrase-window, and destructive scene-window limits: `docs/specs/ghost_api_spec.md:288`.
- Destructive action classification is explicit and bounded to current Action Lexicon commands: `docs/specs/ghost_api_spec.md:305`.
- Ghost API v1 requirements still exclude autonomous performance mode and hidden multi-scene orchestration: `docs/specs/ghost_api_spec.md:372`.
- The roadmap deliverables for Phase I are tool schema, budgets/locks, watch/assist flows, and explainable logs: `docs/execution_roadmap.md:185`.
- Phase 7 done criteria require useful suggestions, approved quantized execution, lock/budget respect, explainable logging, and replay/undo alignment: `docs/phase_definition_of_done.md:141`.

## Implementation Review

### Current Strengths

- **Normal queue boundary preserved**: accepted suggestions go through `queue_accepted_ghost_suggestion(...)` and enqueue a normal action only after mode, blocker, and budget checks: `crates/riotbox-app/src/jam_app/ghost_queue.rs:86`.
- **No hidden Ghost commit path**: accepted actions are appended to `ActionLog` only after normal queue commit, and commit metadata is recorded in `ActionCommitRecord`: `crates/riotbox-app/src/jam_app/commit.rs:103` and `crates/riotbox-core/src/session/version_types.rs:383`.
- **Budget checks are explicit**: pending, phrase, and destructive scene gates reject before proposal acceptance is marked durable: `crates/riotbox-app/src/jam_app/ghost_queue.rs:115`.
- **Source-backed suggestion is honest**: the current automatic Jam-state suggestion is W-30 capture-oriented and requires source evidence plus no existing captures or pending work: `crates/riotbox-app/src/jam_app/ghost_candidates.rs:61`.
- **Destructive classification is visible**: destructive budget enforcement uses an explicit command list rather than inference from labels or result strings: `crates/riotbox-app/src/jam_app/ghost_queue.rs:248`.

### Residual Risks

- **Suggestion breadth is intentionally narrow**: only source-backed capture is generated automatically from Jam state today. That is enough for bounded Watch / Assist MVP, but not a full Ghost product personality.
- **Replay is structurally supported, not a full hardening pass**: accepted Ghost actions use the normal log and commit metadata seam, but deterministic long-run replay remains a Pro Hardening concern.
- **Undo remains action-family dependent**: Ghost actions inherit normal action behavior. Actions whose user path is only partially undoable remain only partially undoable when accepted through Ghost.

## Exit Decision

P010 can be considered closed for bounded Ghost Watch / Assist MVP after the recent safety slices:

- proposal-to-Jam view contract
- stable Ghost tool registry ids
- pending Ghost action budget
- phrase-window Ghost action budget
- destructive scene-window Ghost budget

The next honest step is not autonomous `perform`; it is either:

- Pro Hardening work that proves replay/undo under longer sessions, or
- a bounded product slice that makes Ghost suggestions broader while reusing the same proposal, queue, budget, and commit metadata seams.

