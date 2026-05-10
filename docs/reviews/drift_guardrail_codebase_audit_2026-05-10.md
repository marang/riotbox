# Drift Guardrail Codebase Audit

Date: 2026-05-10

Scope: repo-root review, prioritized through `docs/reviews/riotbox_drift_guardrails_2026-05-10.md`.

## Review Lens

This audit focused on the current drift guardrail yellow zones:

- `JamAppState` and `jam_app*` orchestration growth
- repeated queue-draft construction
- repeated side-effect log-result mutation
- string labels that control behavior
- broad imports and textual `include!` shells
- feature claims that would need real state, replay, observer, or audio proof

This is a review artifact, not a runtime change.

## Summary

Riotbox is in a healthier state than the earlier large-file list suggested. The large Rust files have mostly been split below the soft 500-line review budget, and the splits are mostly semantically named. The remaining risk is not raw file size; it is repeated action plumbing across `jam_app`, replay, UI summaries, and audio-facing render labels.

The current architecture still has a real product spine: typed `ActionCommand`, queue/commit semantics, Session state, replay executor support, Source Graph timing contracts, and audio QA harnesses. The highest-value cleanup should be small and behavior-preserving, aimed at making future action additions harder to get subtly wrong.

## Findings

### 1. TR-909 side effects update lane state but do not consistently record action results

- Location: `crates/riotbox-app/src/jam_app/side_effects/tr909.rs:39`
- Location: `crates/riotbox-app/src/jam_app/side_effects/tr909.rs:48`
- Location: `crates/riotbox-app/src/jam_app/side_effects/tr909.rs:58`
- Location: `crates/riotbox-app/src/jam_app/side_effects/tr909.rs:71`
- Location: `crates/riotbox-app/src/jam_app/side_effects/tr909.rs:84`
- Category: scope
- Severity: major
- Title: TR-909 commit proof is weaker than MC-202 and W-30

`Tr909FillNext`, `Tr909ReinforceBreak`, `Tr909Takeover`, `Tr909SceneLock`, and `Tr909Release` mutate Session lane state, but only `Tr909SetSlam` writes an `ActionResult`. W-30 and MC-202 side-effect modules write musician-facing action results for their committed lane gestures, and the new drift guardrail expects committed action paths to account for the user-visible/observer surface.

The result is not a second action system, but it is a proof-surface gap: a TR-909 gesture can commit and alter state while the log/result surface remains less explicit than adjacent lanes. Existing tests show strong state coverage for TR-909, but result-summary assertions are concentrated on slam and W-30 paths rather than the full TR-909 gesture family.

Suggestion: add a small shared log-result helper and apply it to the TR-909 side-effect module. Cover fill, reinforce, takeover, scene lock, and release with focused result-summary tests. Keep this behavior-preserving: update the already-committed log entry, do not create new log events.

### 2. Queue-draft construction is repeated enough to warrant a narrow helper

- Location: `crates/riotbox-app/src/jam_app/mc202_queue.rs:93`
- Location: `crates/riotbox-app/src/jam_app/mc202_queue.rs:118`
- Location: `crates/riotbox-app/src/jam_app/tr909_queue.rs:101`
- Location: `crates/riotbox-app/src/jam_app/tr909_queue.rs:132`
- Location: `crates/riotbox-app/src/jam_app/w30_queue.rs:101`
- Location: `crates/riotbox-app/src/jam_app/w30_queue.rs:153`
- Location: `crates/riotbox-app/src/jam_app/w30_queue.rs:241`
- Location: `crates/riotbox-app/src/jam_app/w30_queue.rs:444`
- Category: scope
- Severity: major
- Title: Action draft construction can drift between lanes

The lane queue modules repeatedly construct the same shape: build `ActionDraft`, set actor/command/quantization/target, attach params/explanation, enqueue, then refresh the view. That repetition is still readable, but it now crosses the guardrail threshold where the same shape appears in many lane paths.

The current code is not wrong, and a generic factory would be worse. The risk is future drift: a new action can accidentally miss a target scope, quantization, params target, or refresh while looking similar to the neighboring functions.

Suggestion: extract one or two narrow helpers, probably inside `jam_app`, that keep lane intent explicit. Prefer a helper that receives `ActionCommand`, `Quantization`, `TargetScope`, optional bank/pad/object id, params, and explanation, then enqueues and refreshes. Do not hide musical intent behind macros or a broad generic action builder.

### 3. MC-202 role and phrase semantics still depend on behavior-controlling strings

- Location: `crates/riotbox-app/src/jam_app/mc202_queue.rs:30`
- Location: `crates/riotbox-app/src/jam_app/mc202_queue.rs:93`
- Location: `crates/riotbox-app/src/jam_app/side_effects/mc202.rs:31`
- Location: `crates/riotbox-app/src/jam_app/side_effects/mc202.rs:171`
- Location: `crates/riotbox-core/src/replay/executor.rs:220`
- Location: `crates/riotbox-core/src/replay/executor.rs:242`
- Location: `crates/riotbox-core/src/session/version_types.rs:186`
- Location: `crates/riotbox-audio/src/mc202/render_types.rs:1`
- Category: scope
- Severity: major
- Title: MC-202 has typed render concepts but stringly app/session action semantics

MC-202 render modes and phrase shapes are typed in audio code, and the Session model already has a typed `Mc202PhraseVariantState` for one variant. The app and replay path still use strings like `leader`, `follower`, `answer`, `pressure`, `instigator`, and `mutated_drive` as behavior-controlling values in `ActionTarget::object_id`, `ActionParams::Mutation.target_id`, and `SessionFile` lane state.

Those strings are not just display labels; they affect queue decisions, commit side effects, replay reconstruction, and render projection. That matches the drift guardrail definition of suspicious stringly state.

Suggestion: introduce a typed core/session-level MC-202 role and phrase-shape/intent contract, with string labels only at serialization/display boundaries. This should be a bounded migration, not a wholesale rewrite: start by adding typed conversion helpers and tests proving queue, side effects, replay, and render projection agree for the existing roles.

### 4. Textual `include!` shells remain acceptable but should not become permanent architecture

- Location: `crates/riotbox-app/src/ui.rs:1`
- Location: `crates/riotbox-core/src/source_graph.rs:3`
- Location: `crates/riotbox-core/src/view/jam.rs:1`
- Category: scope
- Severity: minor
- Title: Included shards have semantic names, but ownership is still textual

The current `include!` shells are documented and mostly semantically named. They reduce review cost compared to the prior monolithic files and are acceptable under the guardrail because they were behavior-preserving splits.

The remaining risk is that future contributors may treat textual inclusion as durable module ownership. It is not. Real module boundaries should happen only where visibility, tests, and semantic ownership become clearer.

Suggestion: do not create a cleanup ticket just to remove `include!`. Instead, convert a shell to real modules only when a feature or refactor needs a clearer public/private boundary anyway.

## Recommended Follow-Up Slices

1. Add TR-909 committed-result summaries for all TR-909 lane gestures.
2. Extract a narrow app-lane queue draft helper and migrate one lane family first.
3. Add a typed MC-202 role/phrase intent migration plan or first conversion helper.

Do not start all three at once. The first slice is the most immediately useful because it closes a proof-surface gap without broad architecture churn.

## Non-Findings

- No new second queue, second session model, or hidden Feral-only architecture was found in this pass.
- Large Rust files are currently near the soft budget rather than wildly above it. Further splitting should be driven by semantic boundaries, not line count.
- Test-local `use super::*` imports are widespread, but the current production app-module risk is lower than the previous import-boundary audits. Keep watching it, but do not spend a standalone cleanup slice on test imports.
