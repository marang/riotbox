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
- A nearest offline render seam is partial evidence; product-facing instrument
  progress requires the exact live runtime / mixer path when that path is in
  scope.
- Fixture verdicts never count as human listening coverage.
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
- Use `docs/engineering/module_policy.md` as the canonical module and textual
  include policy. File size is a review signal, not a reason to create
  arbitrary shards.
- Do not add `JamAppState` state unless it is truly app-runtime state, does not need restore/replay, and has no better home in Session/Core.

## Work Classification

Classify P023 work before implementation:

- `audible_vertical_slice`: prove the musician action, exact product path,
  audible consequence, and listening state
- `contract_enabler`: name exactly one directly enabled audible follow-up issue
  and outcome
- `maintenance/regression`: preserve behavior and do not claim musical or
  instrument progress

Do not chain contract enablers without landing the named audible follow-up.
Offline binaries, scripted packs, fixtures, reports, and validators remain
diagnostic until the behavior is promoted into product ownership.

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
- Before every human playback, technically analyze and correctly assign the
  exact WAV/A-B artifact, interpret role-appropriate time/frequency deltas, and
  only then give the factual brief and request fresh readiness.
- Bound human playback to 10 seconds by default and 2-5 seconds for an isolated
  capture/stem when sufficient. Exceed 10 seconds only for a named multi-bar
  development claim after stating its exact duration and purpose. Explicitly
  stop and verify silence at the announced endpoint; terminate a live runtime
  immediately if transport stop leaves any lane audible.
- For multi-stage live reviews, queue ahead of intended quantized boundaries
  and validate the landed observer commit intervals plus final stop position
  before requesting readiness. Wall-clock sleeps alone are not timing proof;
  reject a take that misses its declared beat/bar arc.
- Do not ask for repeated human playback when only technical timing, capture,
  or observer evidence changed and the sound recipe did not. Prove those
  changes mechanically; treat repeated "same" or listener fatigue as a stop
  signal, not as a negative musical verdict.
- After at most two consecutive generations of a review-ready candidate that
  remains `human_verdict: unverified`, stop generation for that candidate and
  perform or explicitly hand off structured human listening. Do not replace the
  missing verdict with another report/validator layer unless the failure is
  genuinely unobservable.
- When fuller harnesses land, tighten this rule to the spec's stronger release gates.

## Musical Direction

- Shape Riotbox toward aggressive sample-based rave/punk and breakbeat instrument behavior, not generic EDM preset browsing.
- Use The Prodigy's full-era production arc as a quality reference class, not a
  copy target: early rave break urgency, mid-era big-beat/punk attack, later
  denser bass/drum pressure, vocal/stab hooks, harsh stops, and live-room
  impact. Riotbox output must stay original, source-backed, and its own
  identity, but the quality bar is uncompromising pressure and stage usefulness.
- Prefer short, forceful loops with a memorable hook, stab, riff, break, bass gesture, vocal hit, or silence cut.
- Establish one supported Golden Path with a clear hook and hardest element
  before widening source-family coverage or multiplying variations.
- Require that Golden Path to develop across the listening window: a
  near-identical short loop repeated for eight bars fails unless the mode
  explicitly promises a held loop and its hook already has a human pass.
  Micro-dropouts do not count as macro development; prefer a source-derived
  hook, pressure lift, destructive role swap or drop, and changed return.
- Do not treat a compact scripted performance arc as loop or instrument proof.
  A sequence that forces hook, lift, fill, scene change, and return every one or
  two bars may prove gesture reachability while still sounding like a crowded
  medley. Claims about reusable material or performer freedom additionally need
  a sustained isolated component audition where the musician can keep or reject
  the hook, capture, lane, or mutation before combining roles on demand.
- Before labeling playback as an isolated component, enumerate every audible
  callback contribution. Monitor layers, internal resample taps, diagnostic
  voices, support lanes, and stopped-preview voices either must be silent or
  must be named as part of a composite; an internal route is still audible.
- Do not confuse a cleaner, louder, darker, or busier render with recognizable
  Riotbox character. The character claim requires a memorable source-backed
  hook, physical pressure, dramatic contrast, and stage-useful playability.
- Make live gestures audibly dramatic: trigger, mute, choke, retrigger, reverse, pitch dive, filter slam, bitcrush, fill, and dropout should change the room immediately.
- Do not implement hardcoded musical/audio fallback output as a product path.
  When source-backed generation cannot produce trusted material, surface
  unavailable / degraded state to the musician instead of playing synthetic
  replacement music. Diagnostic controls may compare against silence or
  explicitly labeled non-product controls, but fallback sound must not exist on
  Riotbox product output paths.
- Fixed, typed, versioned instrument vocabulary may be performer-triggered
  product output when an explicit committed action owns it. It must be labeled
  `primitive_renderer`, must not activate as a missing-source fallback, and must
  not claim source-derived selection, composition, or musical intelligence.
  Product-path QA must record the versioned primitive schema, actual recipe ID,
  typed selection inputs, an activation reference resolving to the committed
  command/action/boundary, affected RuntimeMix paths, and declared affected
  artifacts. The shared listening-manifest validator must explicitly register
  every product-output primitive schema and enforce its exact recipe/input,
  source-modulation, activation, RuntimeMix, focus-path, and candidate-WAV
  contract; identifiers that merely look versioned remain diagnostic-only. A
  fixed recipe may remain `primitive_renderer` while source evidence modulates
  its pressure or timbre, but that modulation must be declared separately with
  actual values and must never be mislabeled `availability_and_timing_only`.
- Treat repeated placeholder tones, fallback-only output, polite loops, and technically correct but hookless demos as product failures unless they are explicitly non-product diagnostic controls.
- For every lane or product surface that claims to be source-derived, require
  evidence that Riotbox listened to source features, made a musical decision,
  stored that decision in the product spine, rendered audible output, and proved
  same-source stability plus cross-source diversity.
- During Golden Path tuning, run each shared audible DSP, mix, pattern, or
  performance-policy change against at least five real sources spanning at
  least four typed source families before requesting another human review.
  Several variants from one loop pack or dense-break family count as one
  family. Reserve at least two different-family holdouts that did not choose
  the current algorithm or constants; if a holdout informs the next change it
  becomes a development case and must be replaced with fresh material. The
  Golden Path remains the taste target; the matrix rejects overfitting,
  hot/silent paths, timing regressions, near-identical hook envelopes, and
  holdout failures, but never replaces the human verdict.
- Hardcoded, scripted, or template-mutated output may be useful as a regression
  scaffold, but it is not quality proof and must be labeled accordingly in
  manifests, PRs, specs, and demos.
- Tie taste critique to one concrete implementation or QA follow-up: sample transform, drum policy, trigger behavior, preset change, fixture, threshold, or UI cue.
- State the typed bass owner before judging bass pressure. Relative low-band
  share alone is insufficient; an assigned bass lane needs absolute low-band
  energy or lift, while `unassigned` must not be failed for absent bass.
- Never describe a listening target as generic `pressure`. Distinguish
  bass/low-end, drum/transient, midrange/hook, and arrangement/performance
  impact, and do not let pressure in one domain conceal failure in the intended
  domain.
- Do not equate `Hard` with louder peaks, shorter gates, more silence, clipping,
  brightness, or waveform delta. Transient Hard needs a source-local perceptual
  attack, onset-local spectral or strength change, enough retained body for its
  role, stable timing, and useful performer contrast. Treat roughness and
  arrangement space as separate ingredients. After a fixed gate/gain/drive
  recipe fails listening, change and document the causal mechanism before
  tuning another magic number. A slice-cursor discontinuity is not source
  attack; prime edge history from the selected local source neighborhood and
  declare the bandwidth of any proxy used to judge attack.
- Keep commercial reference recordings local, ignored, and uncommitted. Use
  them only for listening and measurement comparison, never as Riotbox product
  sources, fixtures, generated assets, or redistributed material.
- Positive demo families need real human musical passes. Weak or bad-timing
  sources may satisfy their product contract through reviewed degraded /
  unavailable / reject behavior instead of forced demo-ready music.

## Rust Guidance

- Keep core types explicit and boring.
- Keep tests close to the modules they validate.
- Avoid unnecessary dependencies during early model stabilization.
- For Rust reviews, use `code-review`; Rust context should also apply `code-review-rust` when available.
- Detailed Rust file-size, module-split, textual-include, and test-organization
  rules live in `docs/engineering/module_policy.md`,
  `docs/specs/rust_engineering_guidelines.md`, `docs/workflow_conventions.md`,
  and the `code-review-rust` skill.

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
  - growth of any Rust file beyond the soft review-size guidance, especially
    when a semantic module split would reduce review risk
- Run `review-codebase` regularly, not on every branch.
- Default broad-review cadence: after every 5th substantive feature branch or
  at an active phase checkpoint. Docs-only, archive-only, fixture-only, and
  mechanical maintenance branches do not advance the counter unless they
  materially change architecture or product contracts.
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
- While P023 is active, prefer a waiting structured human review, then the next
  unblocked audible Golden Path slice. Select P016/P021/P022 work only when its
  issue names the exact P023 blocker it removes.

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
