# AGENTS.md

## Purpose And Authority

Use this file as the always-loaded operating kernel for Riotbox coding agents.
Keep implementation aligned with `docs/`, `plan/`, Linear, and Git history; when
implementation and planning diverge, update the owning contract or decision log
instead of inventing behavior silently.

Project-owned skills under `.codex/skills/` are canonical. Do not maintain
separate Riotbox skill copies under `$HOME/.codex/skills`; a home-directory
symlink is only a compatibility fallback for runtimes that cannot discover the
project skills.

This file owns only always-on guardrails and routing. Detailed rules belong to
the first-hop document named below; entry skills may repeat a short safety pin
but not schema fields, numeric gates, command catalogs, or long procedures.

## Always-On Product Invariants

- Preserve the product spine: Source Graph, Session model, Action Lexicon, and
  queue / commit semantics.
- Do not create shadow systems: no second action system, persistence model,
  replay truth, arrangement model, Ghost-only architecture, or Feral-only
  architecture.
- Keep replay-, restore-, capture-lineage-, source-timing-, and product-contract
  state in Core/Session models, not hidden app-local state.
- Treat `JamAppState` as an app facade. Add state there only when it is truly
  runtime-local, needs no restore/replay, and has no better Core/Session owner.
- Keep `feral_rebuild` a profile or policy layer, never a product fork.
- Prefer explicit typed contracts over strings that control branching, replay,
  restore, QA, generated artifacts, or cross-module behavior.
- Keep realtime audio isolated from blocking I/O, analysis, Ghost reasoning,
  heavy UI work, and model calls.
- Never revert unrelated user changes.

Every new `ActionCommand` must account for all five surfaces:

1. Queue path.
2. Commit or side-effect path.
3. Session and replay consequence.
4. User-visible or observer surface.
5. Test and QA proof.

If one is intentionally inapplicable, record why in the PR or working notes.

## Audio, Source, And Human Safety

- Do not close an audio-producing slice from UI, log, or state proof alone. If
  sound should change, prove the output path; when the live mixer is in scope,
  a nearest offline seam is only partial evidence.
- A source-derived claim requires real source evidence to change a musical
  decision represented in the product spine, produce an audible consequence,
  and pass the applicable reproducibility, diversity, and listening gates.
- Hardcoded phrases, templates, scripted demos, fixed diagnostics, and
  fingerprint-only variation are scaffolds or controls, not product
  intelligence.
- Never use hardcoded musical output as an automatic missing-source fallback.
  Fail closed to visible unavailable/degraded state or silence.
- Fixed typed versioned vocabulary is product output only when an explicit
  committed performer action owns it and the audio-QA contract validates it.
  It remains `primitive_renderer`, never fallback or source-intelligence proof.
- Never open, read, hash, render, classify, or play active holdout audio unless
  the exact authorized phase contract permits it. Do not discover source
  directories for holdout work. Once holdout evidence changes implementation,
  record it as consumed and rotate it before claiming fresh evidence again.
- Commercial references stay local, ignored, and uncommitted. They are only
  listening/measurement references, never product sources, fixtures, generated
  assets, review-pack content, or redistribution material.
- Fixture verdicts and automated metrics never count as human listening.
- Before the first human playback of an artifact, apply the listening-review
  skill: preflight the exact artifact, obtain fresh explicit readiness, bound
  playback, stop audibly at the announced endpoint, and verify silence. When
  the listener explicitly requests `again`, `nochmal`, or an equivalent direct
  replay of the unchanged artifact, replay it immediately without repeating
  analysis, the brief, or readiness. Still verify stop/silence. A changed
  artifact requires the full gate again. Terminate a live runtime immediately
  if transport stop leaves any lane audible.
- Translate informal human listening comments into neutral professional
  evidence language in durable documentation while preserving their meaning,
  certainty, and severity. Use verbatim wording only when the exact wording is
  material. Do not rewrite already hash-bound review evidence solely for
  editorial normalization; normalize its meaning in the durable record.
- An isolated-audio claim requires every callback/mixer contributor to be
  silent or named as part of a composite.

## Skill Routing

- Use `riotbox-development` for all Riotbox implementation work.
- Also use `riotbox-rave-punk-production` when work affects audible character,
  patterns, slices, loops, presets, demos, drums/bass, performance controls, or
  musician-facing taste.
- Also use `riotbox-listening-review` before human playback, review-pack work,
  or any musician-facing verdict.
- Use `code-review` before a finished feature branch reaches PR; add
  `code-review-rust` for Rust diffs or crate/module changes.
- Use `review-codebase` at the cadence defined in the workflow contract.
- When a recurring product failure or better QA pattern appears, update its
  canonical skill or repo contract, re-read it, and avoid mirroring the detail
  elsewhere.

## Contract Routing

Start at `docs/README.md` and read only the relevant contract:

- roadmap and completion: `docs/execution_roadmap.md`,
  `docs/phase_definition_of_done.md`, `docs/prd_v1.md`
- product spine and replay: `docs/specs/source_graph_spec.md`,
  `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`,
  `docs/specs/replay_model_spec.md`
- audio runtime: `docs/specs/audio_core_spec.md`
- audio evidence, holdouts, primitives, and listening:
  `docs/specs/audio_qa_workflow_spec.md`
- source timing and scenes: `docs/specs/source_timing_intelligence_spec.md`,
  `docs/specs/arrangement_scene_system_spec.md`
- TUI, Ghost, and presets: `docs/specs/tui_screen_spec.md`,
  `docs/specs/ghost_api_spec.md`, `docs/specs/preset_style_spec.md`
- Rust, modules, and stack: `docs/engineering/module_policy.md`,
  `docs/specs/rust_engineering_guidelines.md`,
  `docs/specs/technology_stack_spec.md`
- typed hardness: `docs/engineering/percussive_force_and_beat_impact.md`
- GitHub, Linear, PR/CI, review, archive, and closeout:
  `docs/workflow_conventions.md`
- sandbox, host audio, and user-session behavior: `docs/dev_environment.md`

Use `plan/` for strategy and history, not as a competing implementation
contract. Query `docs/research_decision_log.md` with `just decision-search`, an
exact `rg`, or a targeted range; do not load it wholesale.

## Workflow Pins

- Normal implementation follows Linear issue -> branch -> PR -> CI/review ->
  merge -> sync `main` -> closeout.
- Linear is first: select exactly one issue and move it to `In Progress` before
  creating or reusing its implementation branch. Autonomous continuation never
  permits backfilling Linear after code or merge work.
- Open PRs and running/green CI gate that PR; when the user explicitly requested
  cross-ticket continuation, they do not pause the next clean bounded slice.
- Keep issue state, priority, labels, project, archive, deletion, and branch
  cleanup aligned. Archive completed/canceled context before deletion.
- While P023 is active, prefer a waiting structured human review, then the next
  unblocked audible Golden Path slice. Other phase work must name the exact P023
  blocker it removes.
- For a new audible mechanism, use the bounded Development-exploration stage in
  `docs/plans/p023_audible_delivery_course_correction.md` before product
  qualification. Exploration cannot grant product, source-general, release,
  hardness, or Holdout claims and must honor that plan's stopping rule.

## Engineering And Review

- Keep contracts explicit and boring; prefer small enums and structs.
- Prefer explicit imports. Follow `module_policy.md`; file size is a review
  signal, mechanical `include!` shards are not durable module ownership, and a
  split is valid only when semantic ownership and review cost improve.
- Keep tests near the behavior they validate and avoid unnecessary dependencies.
- Freeze new technical decisions in the decision log and update the owning spec
  when a contract changes; do not bury architecture decisions only in comments,
  Linear, or chat.
- Append every new `RBX-*` Decision Log entry at end-of-file in monotonically
  increasing ID order; never insert it through a repeated generic field anchor.
- Keep the Decision Log for durable research, architecture, algorithm,
  threshold, access-boundary, and product-contract decisions. Routine playback,
  readiness, replay, status, and other reversible operational steps do not get
  Decision Log entries.
- Branch review must cover correctness, architecture drift, missing tests,
  workflow/docs gaps, and risky Rust/module growth. Fix real findings before PR
  when feasible, then perform a short self-review.
- Treat sandbox-only audio failures as inconclusive until checked in the real
  user session; record which execution context produced the result.

## Context And Commands

- Let normal `rg` respect `.rgignore`. Search archives, generated artifacts, or
  ignored local material only when the task requires them.
- Keep command output bounded. Use `scripts/run_compact.sh /tmp/name.log ...`
  for noisy validation and report status plus relevant tail/error lines.
- Do not paste large manifests, WAV metadata, archives, or transcripts unless
  directly required.
- Use `just --list` and `Justfile` as the command catalog. Prefer `just ci` before
  opening or updating a PR and apply the audio-QA commands required by the
  active contract.
- Riotbox's core/runtime/TUI/audio implementation remains Rust as frozen in the
  technology-stack spec; do not replace the main core with Go.

## When In Doubt

- Prefer the smaller explicit model, replayable contract, realtime boundary,
  source-backed behavior, and documented decision.
- Ask one concise question only when user feedback would materially change the
  immediate implementation; otherwise take the smallest safe in-scope step.
