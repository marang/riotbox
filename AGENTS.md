# AGENTS.md

## Purpose

This repository is in the transition from planning to implementation.

Use this file as the local operating brief for coding agents working in the repo.

The goal is to keep implementation aligned with the planning documents and to prevent architecture drift during the first build slices.

---

## Current State

- Planning and spec layer exists under `docs/` and `plan/`
- Rust workspace is active at the repo root with real implementation across:
  - `crates/riotbox-core`
  - `crates/riotbox-app`
  - `crates/riotbox-audio`
  - `crates/riotbox-sidecar`
- The current product spine is already beyond a minimal shell:
  - Source Graph and Session v1 are real
  - queue / commit semantics and action history are real
  - `Jam`, `Log`, `Source`, and `Capture` shells are real
  - TR-909, MC-202, W-30, and Scene Brain slices already exist behind the current Jam workflow
- The current active roadmap phase should be read from the live docs, not inferred only from this file:
  - `docs/README.md`
  - `docs/execution_roadmap.md`
  - `docs/phase_definition_of_done.md`

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
9. `docs/specs/validation_benchmark_spec.md`
10. `docs/specs/fixture_corpus_spec.md`
11. `docs/specs/audio_qa_workflow_spec.md`
12. `docs/specs/tui_screen_spec.md`
13. `docs/specs/ghost_api_spec.md`
14. `docs/specs/preset_style_spec.md`

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

### Review gate

- Before committing a finished feature-branch slice, run the `code-review` skill on the branch diff when that skill is available in the current session.
- Do not assume one hardcoded user path for skills. If a skill path from session context is missing, check `$HOME/.codex/skills/<skill>/SKILL.md` before falling back.
- Use that review to identify findings, fix them on the branch, and answer any review questions before opening the PR.
- After that, still run a short self-review on the branch diff.
- The branch-level review should explicitly check for:
  - correctness bugs
  - architecture drift against `docs/` contracts
  - missing tests for new behavior
  - workflow/documentation gaps introduced by the slice
- If the review finds a real issue, fix it on the branch before creating the PR when feasible

### Audio-producing slices

- For audio-producing changes, treat `docs/specs/audio_qa_workflow_spec.md` as an active workflow contract, not only an indexed reference.
- Use it to decide which current audio QA layers apply to the slice and which ones are still intentionally not operational in the repo yet.
- Do not claim offline WAV review packs, candidate-vs-baseline audio directories, or formal listening-pack gates unless the slice actually uses a real harness that already exists in the repo.
- Until the fuller audio QA harnesses land, the minimum expectation for audio-producing slices is:
  - relevant unit and integration tests
  - relevant buffer regression coverage when the slice touches an existing audio seam
  - local manual listening against the real session when the behavior changed materially and can be heard today
  - explicit notes in the PR or working context when an audio QA layer from the spec is still aspirational rather than operational
- When the fuller harnesses do land, tighten this rule to use the spec's stronger release gates instead of treating them as future work.

### Periodic codebase review

- Run the `review-codebase` skill on a regular cadence, not on every feature branch.
- Default cadence: after every 5th finished feature branch.
- Use that broader review to catch cross-slice drift, recurring weaknesses, missing tests, and architecture erosion that branch-local review may miss.
- Record important findings in:
  - `docs/reviews/`
  - `docs/research_decision_log.md`
  - workflow/docs updates when the findings change how the repo should be operated
- If the `review-codebase` skill is not available in the current session, note that explicitly and fall back to a normal whole-codebase review pass.

### CI gate

- After opening a PR, inspect the GitHub Actions / CI results explicitly.
- Do not treat a ticket as merely "waiting for review" if CI is red.
- If CI fails and the failure is relevant to the slice, fix it on the same branch before considering the review boundary clean.
- Treat open PRs and in-flight CI as merge gates, not as a reason to pause the main implementation lane.
- If the current PR is locally clean and CI is only running or already green, continue with the next bounded roadmap-aligned slice and re-check the open PR periodically.
- When no event or webhook mechanism is available, use explicit periodic polling instead of idling.
- Do not emit standalone status-only progress reports when there is no real blocker.
- If a user-facing update is necessary during active work, tie it directly to the next concrete action already being taken.
- At minimum, check:
  - formatter status
  - test status
  - lint status
  - any slice-specific workflow required by the repo

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
- Follow the repo workflow note in `docs/workflow_conventions.md` for branch / PR / merge / Linear conventions.
- Keep Linear priorities explicit:
  - `In Progress` / `In Review` -> `High (2)`
  - honest near-next backlog -> `Medium (3)`
  - distant work -> `Low (4)` or unset
  - archive / repo-ops slices -> usually `Medium (3)` unless urgent
- Keep Linear labels explicit and orthogonal to projects:
  - projects answer phase
  - labels answer slice type
  - current base labels:
    - `workflow`
    - `archive`
    - `ux`
    - `benchmark`
    - `review-followup`
- Treat workflow and archive obligations as a real work lane, not as optional cleanup after coding.
- When delegation is available and the slice is substantial enough, prefer two parallel lanes:
  - main implementation lane
  - workflow / ops lane or subagent for Linear state, project updates, archive prep, and similar repo-process obligations
- The workflow / ops lane should keep Linear and repo bookkeeping continuously aligned while implementation is moving, not only after the code is already finished.
- The main thread may keep implementing while the workflow / ops lane keeps Linear state, project updates, and archive obligations aligned.
- The main thread still owns correctness, final review, PR quality, merge readiness, and final integration.
- Keep a small active backlog in Linear so work does not stall at ticket boundaries.
- Treat this as an active rule, not a soft preference.
- During active implementation, do not let the working backlog drop to zero when the next likely slice is already clear.
- Before closing the current ticket loop, ensure Linear still has:
  - 1 ticket in progress or in review
  - 1-5 near-next tickets in backlog
  - milestone-level placeholders for later work only when they stay coarse and honest
- Do not fully decompose distant phases into many detailed tickets before nearer slices land.
- Treat Linear as the active operations layer, not the long-term archive.
- Before deleting a completed Linear issue to stay under the free-tier cap, archive its useful context into repo markdown under `docs/archive/linear_issues/`.
- Do that archive work as part of the ticket closeout path, not as a separate default `Archive ...` ticket.
- Use the repo archive as canonical Git-backed history.
- MemPalace should stay focused on live product docs and specs, not archived Linear ticket files.
- For important architecture, review, and decision-heavy tickets, prefer one archive file per ticket.
- For routine feature tickets, a grouped archive file is acceptable when the entries stay readable and searchable.
- Keep archive entries structurally uniform.
- Use:
  - `RIOTBOX-123.md` for one-ticket archive files
  - `YYYY-MM.md` for grouped monthly archive files
- Use ISO dates (`YYYY-MM-DD`) for all archived ticket timestamps.
- Keep metadata fields in the same order as the archive template so entries stay easy to scan, diff, and mine.
- Use stable final-status terms such as:
  - `Done`
  - `Canceled`
  - `Duplicate`
  - `Superseded`
- At minimum, preserve:
  - ticket id and title
  - Linear project
  - milestone or phase
  - ticket status such as done, canceled, or superseded
  - created date
  - implementation start date when known
  - done, merged, canceled, or deleted date when applicable
  - actual repo feature branch used for the work
  - why the ticket existed
  - what shipped
  - PR link
  - merge commit
  - follow-up tickets or bounded open questions
- When useful, also preserve:
  - Linear-generated branch name if it differed from the actual repo branch and is worth keeping
  - Linear issue URL
  - labels
  - assignee or owner
  - deleted-from-Linear date
  - verification summary
  - decision-log or spec links touched by the ticket
- Only delete the Linear issue after the PR is merged, the issue is done, and the repo archive entry exists.
- Prefer the repo-local helper for deletion:
  - `scripts/linear_issue_delete.sh RIOTBOX-123`
- Use token auth for that helper:
  - `LINEAR_API_TOKEN=...`
- Do not rely on pasted browser session cookies as the normal workflow path.

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
- After a ticket is cleanly closed, the agent may autonomously start the next-best backlog ticket if:
  - the previous slice is merged or otherwise fully closed
  - no unresolved review or CI blocker remains on the closed slice
  - the next ticket satisfies the next-ticket heuristic above
  - the near-term Linear backlog still stays within the repo rule of 1-5 honest backlog tickets
- Prefer continuing through the next-best roadmap-aligned ticket instead of waiting for a user nudge when the next step is already clear and bounded.

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
just w30-smoke-qa local
cargo run -p riotbox-audio --bin w30_preview_render
cargo run -p riotbox-audio --bin w30_preview_compare
scripts/linear_issue_delete.sh RIOTBOX-123
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

## Agent Sandbox Howto

When Riotbox runs inside `agent-sandbox`, do not guess which host capability is missing. Check it explicitly and then ask for the smallest missing mount or image capability.

First-line self-checks:

```bash
command -v git
command -v cargo
command -v pkg-config
command -v podman
pkg-config --libs --cflags alsa
test -S "/run/user/$(id -u)/podman/podman.sock" && echo podman-socket-ok
```

Interpretation:

- if `pkg-config --libs --cflags alsa` fails, the sandbox cannot build the current Linux audio path cleanly
- if `podman` is missing or the Podman socket is unavailable, MemPalace operational commands cannot run from inside the sandbox
- `just` is convenient but not required; prefer direct script commands if `just` is absent

Preferred solution:

- bake the needed tooling into the sandbox image
- use mounts only for host-specific assets or sockets

### Arch Host Requirements

For the current Riotbox repo on an Arch host, these are the practical requirements.

Audio build requirements:

- `pkg-config` available in the sandbox
- ALSA headers and pkg-config data visible in the sandbox

Useful host mounts on Arch:

- `/usr/include/alsa` -> `/usr/include/alsa`
- `/usr/lib/pkgconfig` -> `/usr/lib/pkgconfig`
- `/usr/lib/libasound.so` -> `/usr/lib/libasound.so`
- `/usr/lib/libasound.so.2` -> `/usr/lib/libasound.so.2`

Required environment:

- `PKG_CONFIG_PATH=/usr/lib/pkgconfig`

MemPalace operational requirements:

- `podman` client available in the sandbox
- `podman compose` support available in the sandbox

If using the host's rootless Podman instead of nested Podman inside the sandbox:

- mount the host Podman socket:
  - `/run/user/<host-uid>/podman/podman.sock`
- expose it at the same path or a known sandbox path
- set:
  - `CONTAINER_HOST=unix:///run/user/<host-uid>/podman/podman.sock`

In that setup, the repo-local `scripts/mempalace.sh` wrapper can use the host container runtime without needing full nested container support.

### Git Push Ergonomics

For smoother Git pushes from the sandbox:

- ensure SSH auth is available
- ensure GitHub host trust is available

The current agent can work around missing writable `known_hosts` by using a temporary file, but a better sandbox setup is:

- writable `~/.ssh/known_hosts`, or
- pre-seeded GitHub host keys inside the image

### Host Services

When the sandbox needs to reach a host-local TCP service, prefer:

- `host.containers.internal`

Do not assume `localhost` refers to the host. In the sandbox it is container-local.

---

## When In Doubt

- Prefer the smaller, more explicit model
- Prefer the contract that preserves replayability
- Prefer the implementation that keeps realtime boundaries clean
- Prefer updating docs over leaving hidden assumptions in code
