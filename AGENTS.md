# AGENTS.md

## Purpose

Use this file as the local operating brief for coding agents working in Riotbox.

Riotbox is transitioning from planning into implementation. Keep implementation aligned with `docs/`, `plan/`, Linear, and Git history. If implementation and planning diverge, update the relevant spec or decision log instead of silently inventing behavior.

The project-owned Codex skills live under `.codex/skills/`; treat those files
as canonical. Do not keep separate Riotbox skill copies under
`$HOME/.codex/skills`. Use home-directory symlinks only as a temporary fallback
for older Codex runtimes that cannot discover project-local skills.

When the local `riotbox-development` skill is available, use it for Riotbox development work. That skill captures the expectation that agents act as senior software engineers, senior audio engineers, and musician-users of the instrument.

When Riotbox work affects audible character, pattern quality, slices, loops, presets, demos, drum/bass behavior, or performance controls, also use the local `riotbox-rave-punk-production` skill when available. That skill captures the aggressive sample-based rave/punk production pressure Riotbox should be shaped by: hard hooks, physical drums, bass pressure, destructive variations, live triggerability, and clear failure modes for polite, generic, or placeholder output.

If Riotbox work reveals a recurring product failure mode or better QA pattern, update that skill, re-read it, and mirror durable project rules into this file or the relevant spec. Keep operational GitHub, Linear, PR, CI, branch, archive, and closeout procedure in `docs/workflow_conventions.md`.

## Critical Rules

- Preserve the product spine: Source Graph, Session model, Action Lexicon, and queue / commit semantics.
- Do not create shadow systems: no second action system, persistence model, replay truth, arrangement model, Ghost-only architecture, or Feral-only architecture.
- Keep realtime audio isolated from blocking I/O, analysis work, Ghost reasoning, heavy UI work, and model calls.
- Represent replay-, restore-, capture-lineage-, source-timing-, or product-contract state in core/session models, not hidden app-local state.
- Do not present source-aware templates, hardcoded phrases, scripted demos, fixed
  diagnostics, or fingerprint-only variation as product intelligence. They are
  scaffolds or controls until source evidence actually changes musical choice
  and output.
- Do not close an audio-producing slice with only UI/log proof. If sound should change, prove the output path.
- For every new `ActionCommand`, account for queue, commit/side-effect, Session/replay, user/observer, and QA surfaces.
- Treat `JamAppState` as an app facade, not a second product truth.
- Run branch-level review before PRs when the `code-review` skill is available.
- Inspect GitHub Actions / CI explicitly after opening a PR.
- Follow the GitHub tooling path in `docs/workflow_conventions.md`; SSH `git`
  alone is not PR creation.
- Keep Linear current: issue state, priority, labels, project, archive, deletion, and branch cleanup are part of the work.
- Use one archive file per Linear ticket under `docs/archive/linear_issues/RIOTBOX-123.md`; month files are indexes only.
- Search archives and generated artifacts only when needed. Default `rg` should respect `.rgignore`.
- Do not read `docs/research_decision_log.md` wholesale during normal implementation work; use `just decision-search "query"`, exact `rg`, or targeted line ranges.
- `just decision-search "query"` is a bounded `rg` helper for the decision log. It has no semantic-memory dependency.
- Keep command output token-bounded. Redirect long CI/QA logs to `/tmp/...log` and report only exit status plus relevant tail/error lines.
- Use `scripts/run_compact.sh /tmp/name.log <command...>` for noisy validation commands unless full output is explicitly needed.
- Never revert unrelated user changes.

## Orientation

- Planning and implementation contracts live under `docs/`; strategy and historical planning live under `plan/`.
- Rust workspace crates: `riotbox-core`, `riotbox-app`, `riotbox-audio`, and `riotbox-sidecar`.
- Read the active roadmap phase from `docs/README.md`, `docs/execution_roadmap.md`, and `docs/phase_definition_of_done.md`.

## Source Of Truth

Use `docs/README.md` as the documentation map. Before structural changes, read only the relevant contracts:

- product and roadmap: `docs/prd_v1.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`
- architecture spine: `docs/specs/source_graph_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/replay_model_spec.md`
- audio/runtime: `docs/specs/audio_core_spec.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/fixture_corpus_spec.md`, `docs/specs/validation_benchmark_spec.md`
- TUI/Ghost/style: `docs/specs/tui_screen_spec.md`, `docs/specs/ghost_api_spec.md`, `docs/specs/preset_style_spec.md`
- Rust/stack: `docs/specs/rust_engineering_guidelines.md`, `docs/specs/technology_stack_spec.md`
- source timing/scenes: `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/arrangement_scene_system_spec.md`

Strategic context lives in `plan/riotbox_masterplan.md` and `plan/riotbox_liam_howlett_feral_addendum.md`. Agent-facing drift guardrails live in `docs/reviews/riotbox_drift_guardrails_2026-05-10.md`.

## Architecture Guardrails

- Keep contracts explicit and boring.
- Prefer small enums and structs over stringly behavior.
- Do not bypass Source Graph, Session, Action Lexicon, or queue / commit semantics.
- Keep `feral_rebuild` as a profile / policy layer, not a product fork.
- If a string controls branching, replay, restore, QA, generated artifacts, or cross-module behavior, turn it into a typed contract or document why it stays a string.
- Prefer explicit imports in app modules.
- Avoid new `use super::*` imports unless the local test/module context keeps dependencies harmless and reviewable.
- Repeated queue-draft construction and side-effect log-result mutation are acceptable while small; review for a narrow helper after the same shape appears across three or more lane paths.
- Mechanical `include!` splits are not durable module ownership.
- Convert included shards into real modules only when semantic boundary, visibility, tests, and review cost all improve.
- Do not add `JamAppState` state unless it is truly app-runtime state, does not need restore/replay, and has no better home in Session/Core.

## ActionCommand Rule

Every new `ActionCommand` must explicitly account for:

1. Queue path.
2. Commit / side-effect path.
3. Session / replay consequence.
4. User-visible or observer surface.
5. Test / QA proof.

If a surface is intentionally not applicable, say why in the PR or working notes.

## Realtime Audio Rules

- Isolate the audio path from blocking I/O, analysis work, Ghost reasoning, heavy UI work, and model calls.
- Treat sandbox-only audio failures as inconclusive until verified against the real user session.
- Distinguish sandboxed execution from real user-session execution.
- For Linux audio validation, record whether the result came from restricted sandbox context or a real user session.
- On this machine, the real session uses PipeWire, while `cpal` can still report and use the Linux `Alsa` host successfully.
- Use real-session verification for audio spikes, device enumeration, and latency checks.

## Audio-Producing Slices

- Treat `docs/specs/audio_qa_workflow_spec.md` as an active workflow contract.
- Use the spec to decide which current audio QA layers apply and which are still aspirational.
- Do not claim offline WAV review packs, candidate-vs-baseline audio directories, or formal listening-pack gates unless the slice uses an existing harness.
- Minimum current proof:
  - relevant unit and integration tests
  - buffer regression coverage when touching an existing audio seam
  - action/log/state assertions proving the intended path landed
  - output assertions proving the seam is not silent, not fallback-collapsed, and inside expected metrics
  - local manual listening when the behavior materially changes and is audible today
  - explicit PR or working-context notes when a stronger audio QA layer is still aspirational
- For PRs that affect audible behavior, state whether a structured listening-review
  pack/verdict exists, or why the change remains `human_verdict: unverified`.
- When fuller harnesses land, tighten this rule to the spec's stronger release gates.

## Musical Direction

- Shape Riotbox toward aggressive sample-based rave/punk and breakbeat instrument behavior, not generic EDM preset browsing.
- Prefer short, forceful loops with a memorable hook, stab, riff, break, bass gesture, vocal hit, or silence cut.
- Make live gestures audibly dramatic: trigger, mute, choke, retrigger, reverse, pitch dive, filter slam, bitcrush, fill, and dropout should change the room immediately.
- Do not implement hardcoded musical/audio fallback output as a product path.
  When source-backed generation cannot produce trusted material, surface
  unavailable / degraded state to the musician instead of playing synthetic
  replacement music. Diagnostic controls may compare against silence or
  explicitly labeled non-product controls, but fallback sound must not exist on
  Riotbox product output paths.
- Treat repeated placeholder tones, fallback-only output, polite loops, and technically correct but hookless demos as product failures unless they are explicitly non-product diagnostic controls.
- For every lane or product surface that claims to be source-derived, require
  evidence that Riotbox listened to source features, made a musical decision,
  stored that decision in the product spine, rendered audible output, and proved
  same-source stability plus cross-source diversity.
- Hardcoded, scripted, or template-mutated output may be useful as a regression
  scaffold, but it is not quality proof and must be labeled accordingly in
  manifests, PRs, specs, and demos.
- Tie taste critique to one concrete implementation or QA follow-up: sample transform, drum policy, trigger behavior, preset change, fixture, threshold, or UI cue.

## Rust Guidance

- Keep core types explicit and boring.
- Keep tests close to the modules they validate.
- Avoid unnecessary dependencies during early model stabilization.
- For Rust reviews, use `code-review`; Rust context should also apply `code-review-rust` when available.
- Detailed Rust file-size, module-split, and test-organization rules live in `docs/specs/rust_engineering_guidelines.md`, `docs/workflow_conventions.md`, and the `code-review-rust` skill.

## Documentation Rules

- Freeze new technical decisions in `docs/research_decision_log.md`.
- Update the corresponding spec in `docs/specs/` when a contract changes.
- Do not bury important architecture decisions only in code comments.
- Keep important workflow rules in repo docs, not only in Linear or chat memory.

## Context Hygiene

- Keep normal searches focused on live source and canonical docs.
- Let default `rg` respect `.rgignore`.
- `.rgignore` excludes long Linear archives, raw planning transcripts, generated artifacts, and local audio data.
- Search `docs/archive/linear_issues/` only when ticket history is needed.
- Search ignored archive or audio paths explicitly with `rg --no-ignore "..." <path>`.
- Do not paste large generated manifests, WAV metadata dumps, archive batches, or raw transcript sections unless directly needed.
- Treat `docs/research_decision_log.md` as a large canonical log: append durable decisions when needed, but query it by `just decision-search`, exact search, or targeted line ranges instead of loading the full file.
- Prefer specific files and line ranges over entire long documents.
- If something matters, write it into repo docs, specs, Linear, or Git history; do not leave it only in chat or local memory.

## Review And QA

- Before committing a finished feature-branch slice, run the `code-review` skill on the branch diff when available.
- If a skill path from session context is missing, check `$HOME/.codex/skills/<skill>/SKILL.md` before falling back.
- Fix real review findings on the branch before opening the PR when feasible.
- Still run a short self-review on the branch diff.
- Branch review must check:
  - correctness bugs
  - architecture drift against `docs/` contracts
  - missing tests for new behavior
  - workflow/documentation gaps introduced by the slice
  - growth of any Rust file beyond the 500-line budget
- Run `review-codebase` regularly, not on every branch.
- Default broad-review cadence: after every 5th finished feature branch.
- Record important broad-review findings in:
  - `docs/reviews/`
  - `docs/research_decision_log.md`
  - workflow/docs updates when the findings change repo operation
- If `review-codebase` is unavailable, note it and do a normal whole-codebase review pass.

## Operating Workflow

Use `docs/workflow_conventions.md` as the canonical operational workflow for
GitHub, Linear, PR/CI gates, branch cleanup, ticket archive/deletion, backlog
horizon, and automatic next-ticket continuation. Keep this file focused on
non-negotiable agent guardrails and avoid restating the full procedure here.

Hard rules that must stay true:

- Normal implementation work uses the Linear issue -> branch -> PR -> CI/review
  -> merge -> sync `main` -> closeout loop.
- Linear is first in that loop: before creating or reusing a feature branch for
  implementation, create or pick exactly one Linear issue for the slice and move
  it to `In Progress`. Autonomous continuation, "do not stop", or "keep
  implementing" never permits backfilling Linear after the branch/PR/merge.
- Open PRs and in-flight CI are merge gates for that PR, not a reason to pause
  the main implementation lane.
- If CI is running or green and the branch is locally clean, continue with the
  next bounded roadmap-aligned slice through the same Linear-first loop.
- Keep Linear state, priority, labels, archive entries, branch cleanup, and
  project updates aligned with the repo workflow.
- Archive completed or canceled Linear context under
  `docs/archive/linear_issues/` before deleting Linear issues; deletion requires
  token-backed Linear auth and must not rely on semantic memory.
- Derive the next ticket from `docs/execution_roadmap.md`,
  `docs/phase_definition_of_done.md`, the active spec, and actual repo state;
  prefer the smallest coherent slice on the product spine.

## Commands

Keep this section as a short command shortlist. Use `just --list` and `Justfile` for the full command catalog. Environment and sandbox notes live in `docs/dev_environment.md`.

Default checks:

```bash
cargo fmt
cargo test
just ci
just audio-qa-ci
just check
just clippy
```

Common development helpers:

```bash
just source-timing-fixture-catalog
just source-timing-wav-probe
just source-timing-readiness-report
just decision-search "source timing"
```

Common audio and user-session probes:

```bash
just w30-smoke-qa local
just w30-smoke-source-qa "data/test_audio/examples/Beat03_130BPM(Full).wav" local
just lane-recipe-pack local 2.0
just feral-before-after "data/test_audio/examples/Beat03_130BPM(Full).wav" local
just feral-grid-pack "data/test_audio/examples/Beat03_130BPM(Full).wav" local 130.0 8 1.0
just p012-all-lane-source-grid-output-proof
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav" --observer artifacts/audio_qa/local/user-session/events.ndjson
```

Workflow helpers:

```bash
scripts/archive_linear_issue.py --ticket RIOTBOX-123 --pr 99 --why "..." --shipped "..."
scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99
scripts/linear_issue_delete.sh RIOTBOX-123
```

Current CI baseline:

- GitHub Actions runs:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Before opening or updating a PR, prefer running `just ci` locally.

## Sandbox And Environment Notes

- Do not assume a failed audio probe inside the sandbox means the machine audio stack is broken.
- Record whether Linux audio validation came from sandbox or real user session.
- Treat sandbox-only audio failures as inconclusive.
- Detailed sandbox, Arch, SSH, and host-service notes live in `docs/dev_environment.md`.

## Stack And Layout

The stack freeze is documented in `docs/specs/technology_stack_spec.md`. Defaults: Rust for core/runtime/TUI/audio, Python later for analysis sidecar, JSON for early persistence, `cpal` for audio I/O, `tokio` for control-plane async, and `ratatui` for terminal UI. Do not replace Rust with Go for the main core.

Important repo paths:

- `crates/riotbox-core`: shared core models and logic.
- `crates/riotbox-app`: app-level orchestration and Jam state wiring.
- `crates/riotbox-audio`: audio runtime and callback-side work.
- `crates/riotbox-sidecar`: sidecar protocol/client work.
- `docs/`: implementation-facing contracts.
- `plan/`: strategy and historical planning material.
- `python/sidecar`: analysis process path.

## When In Doubt

- Prefer the smaller, more explicit model.
- Prefer the contract that preserves replayability.
- Prefer realtime boundaries.
- Prefer docs updates over hidden assumptions.
- Ask one concise clarifying question when user feedback mixes later ideas and immediate implementation requests, unless the intended next action is explicit.
