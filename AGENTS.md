# AGENTS.md

## Purpose

This repository is in the transition from planning to implementation.

Use this file as the local operating brief for coding agents working in the repo.

The goal is to keep implementation aligned with the planning documents and to prevent architecture drift during the first build slices.

---

## Current State

- Planning and spec layer exists under `docs/` and `plan/`
- Initial Rust workspace exists at the repo root
- First real code slice lives in `crates/riotbox-core`
- Current implemented foundations:
  - Source Graph v1 model
  - Session v1 model
  - Action types
  - Action queue
  - minimal Jam view model

---

## Source Of Truth

Read these before making structural changes:

1. `docs/prd_v1.md`
2. `docs/execution_roadmap.md`
3. `docs/specs/technology_stack_spec.md`
4. `docs/specs/rust_engineering_guidelines.md`
5. `docs/specs/source_graph_spec.md`
6. `docs/specs/session_file_spec.md`
7. `docs/specs/action_lexicon_spec.md`
8. `docs/specs/audio_core_spec.md`
9. `docs/specs/tui_screen_spec.md`
10. `docs/specs/ghost_api_spec.md`
11. `docs/specs/preset_style_spec.md`

Strategic context lives in:

- `plan/riotbox_masterplan.md`
- `plan/riotbox_liam_howlett_feral_addendum.md`

If implementation and planning diverge, update the relevant spec or decision log rather than silently inventing new behavior.

---

## Architecture Rules

### 1. Contracts before cleverness

Do not introduce behavior that bypasses:

- Source Graph
- Session model
- Action Lexicon
- queue / commit semantics

### 2. No shadow systems

Do not create:

- a second action system
- a second persistence model
- a second arrangement model hidden behind Ghost
- a separate feral architecture

`feral_rebuild` must stay a profile / policy layer, not a fork of the product.

### 3. Realtime discipline

The audio path must remain isolated from:

- blocking I/O
- analysis work
- Ghost reasoning
- heavy UI work

### 4. Determinism matters

If state affects restore, replay, or capture lineage, it should be represented explicitly in the core model.

---

## Frozen Stack v1

The current stack freeze is documented in `docs/specs/technology_stack_spec.md`.

Use these defaults unless a documented spike disproves them:

- `Rust` for core, runtime-facing state, TUI, and audio path
- `Python` later for the analysis sidecar
- `JSON` for early persistence and inspection
- planned runtime direction:
  - `cpal` for audio I/O
  - `tokio` for control-plane async work
  - `ratatui` for terminal UI

Do not replace Rust with Go for the main core.

---

## Repo Layout

Current important paths:

- `crates/riotbox-core`
  Shared core models and logic
- `docs/`
  Implementation-facing contracts
- `plan/`
  Strategy and historical planning material

Expected near-term additions:

- `crates/riotbox-app`
  app-level orchestration and Jam state wiring
- `crates/riotbox-audio`
  audio runtime and callback-side work
- `python/sidecar`
  analysis process

---

## Near-Term Build Order

Follow this order unless the user explicitly redirects:

1. stabilize core data models
2. add serialization roundtrips for `SourceGraph` and `SessionFile`
3. build app-level Jam state wiring
4. run bounded spikes:
   - audio latency
   - Rust/Python transport
   - deterministic replay
   - session serialization
5. move into core skeleton runtime work

Do not jump to advanced DSP, Ghost `perform`, or export-heavy workflows early.

---

## Coding Guidance

### Rust

- Keep core types explicit and boring
- Prefer small enums and structs over stringly behavior
- Keep tests close to the modules they validate
- Avoid unnecessary dependencies during early model stabilization

### Documentation

- If a technical decision is newly frozen, add it to `docs/research_decision_log.md`
- If a contract changes, update the corresponding spec in `docs/specs/`
- Do not bury important architecture decisions only in code comments

### Git hygiene

- Do not revert unrelated user changes
- Keep commits scoped to one coherent slice where possible

### PR descriptions

- Every PR description should include a short `Why This Matters` section.
- That section should explain the slice in product and roadmap terms, not only list code changes.
- At minimum, state:
  - what larger phase or milestone this slice belongs to
  - what real product path or architecture seam it unlocks
  - what is still intentionally out of scope or stubbed
- Do not write PR descriptions as changelogs only.

### Linear hygiene

- Keep Linear updates human-readable.
- Move issues to review when the PR is open and to done when the PR is merged.
- Archive completed issues once they are truly finished so the workspace stays under the free-tier issue cap.
- Follow the repo workflow note in `docs/workflow_conventions.md` for branch / PR / merge / Linear conventions.

### Next-ticket heuristic

- Derive the next ticket from:
  - `docs/execution_roadmap.md`
  - `docs/phase_definition_of_done.md`
  - the active feature spec for the area being worked on
  - the actual current repo state
- Prefer the smallest coherent slice that closes the most immediate product-path or architecture-path gap.
- Do not define many future tickets in detail before the current slice lands.
- Validate each proposed next ticket against four questions:
  - does it fit the current phase?
  - does it create visible product progress or remove a real blocker?
  - does it preserve the current architecture instead of creating a shadow path?
  - is it small enough to review as one coherent slice?
- If multiple candidates are possible, prefer the one that keeps Riotbox moving along the product spine already defined in the roadmap instead of opening a new side path.

---

## Commands

Current useful commands:

```bash
cargo fmt
cargo test
just ci
just check
just clippy
just mem-status
just mem-search "replay truth"
```

Add new commands here when the repo grows enough that agents need a stable shortlist.

Current CI baseline:

- GitHub Actions runs:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Before opening or updating a PR, prefer running `just ci` locally.

## MemPalace Dev Memory

MemPalace is available as an optional dev-memory helper.

Rules:

- it is not product core
- it is not a source of truth
- canonical project truth still lives in `docs/`, `plan/`, Linear, and Git history
- use it to complement `rg`, not replace it

Repo-local layout:

- `.mempalace/palace/`
  persistent Chroma database
- `.mempalace/cache/`
  model and package cache
- `.mempalace/results/`
  captured evaluation outputs
- `.mempalace/corpus/`
  copied project corpus for mining

Operational path:

- use `just mem-init` for the first setup
- use `just mem-status` and `just mem-search "..."` for normal use
- the wrapper script uses rootless Podman with pinned `python:3.12`
- the wrapper automatically re-mines when `docs/`, `plan/`, `crates/`, or `AGENTS.md` changed

Do not store new canonical decisions only in MemPalace. If something matters, it still needs to be written into repo docs or Linear.

---

## Sandboxed Audio

Audio and device probing require extra care in this environment.

Rules:

- Do not assume a failed audio probe inside the sandbox means the machine audio stack is broken.
- Distinguish clearly between:
  - sandboxed execution
  - real user-session execution
- For Linux audio validation, record whether a result came from:
  - restricted sandbox context
  - escalated command against the live user session
- Treat sandbox-only audio failures as inconclusive until verified against the real session.

Current known behavior from Riotbox work:

- sandboxed audio and session-bus access can fail even when the machine audio setup is healthy
- on this machine, the real session uses PipeWire, while `cpal` can still report and use the Linux `Alsa` host successfully

Practical consequence:

- use real-session verification for audio spikes, device enumeration, and latency checks
- write down which context produced the observation

---

## When In Doubt

- Prefer the smaller, more explicit model
- Prefer the contract that preserves replayability
- Prefer the implementation that keeps realtime boundaries clean
- Prefer updating docs over leaving hidden assumptions in code
