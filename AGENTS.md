# AGENTS.md

## Purpose

Use this file as the local operating brief for coding agents working in Riotbox.

Riotbox is transitioning from planning into implementation. Keep implementation aligned with `docs/`, `plan/`, Linear, and Git history. If implementation and planning diverge, update the relevant spec or decision log instead of silently inventing behavior.

When the local `riotbox-development` skill is available, use it for Riotbox development work. That skill captures the expectation that agents act as senior software engineers, senior audio engineers, and musician-users of the instrument.

If Riotbox work reveals a recurring failure mode, workflow gap, or better QA pattern, update that skill, re-read it, and mirror durable project rules into this file, `docs/workflow_conventions.md`, or the relevant spec.

## Critical Rules

- Preserve the product spine: Source Graph, Session model, Action Lexicon, and queue / commit semantics.
- Do not create shadow systems: no second action system, persistence model, replay truth, arrangement model, Ghost-only architecture, or Feral-only architecture.
- Keep realtime audio isolated from blocking I/O, analysis work, Ghost reasoning, heavy UI work, and model calls.
- Represent replay-, restore-, capture-lineage-, source-timing-, or product-contract state in core/session models, not hidden app-local state.
- Do not close an audio-producing slice with only UI/log proof. If sound should change, prove the output path.
- For every new `ActionCommand`, account for queue, commit/side-effect, Session/replay, user/observer, and QA surfaces.
- Treat `JamAppState` as an app facade, not a second product truth.
- Run branch-level review before PRs when the `code-review` skill is available.
- Inspect GitHub Actions / CI explicitly after opening a PR.
- Keep Linear current: issue state, priority, labels, project, archive, deletion, and branch cleanup are part of the work.
- Use one archive file per Linear ticket under `docs/archive/linear_issues/RIOTBOX-123.md`; month files are indexes only.
- Search archives and generated artifacts only when needed. Default `rg` should respect `.rgignore`.
- Keep command output token-bounded. Redirect long CI/QA logs to `/tmp/...log` and report only exit status plus relevant tail/error lines.
- Never revert unrelated user changes.

## Current State

- Planning and spec layers exist under `docs/` and `plan/`.
- Rust workspace is active at the repo root:
  - `crates/riotbox-core`
  - `crates/riotbox-app`
  - `crates/riotbox-audio`
  - `crates/riotbox-sidecar`
- The product spine is beyond a minimal shell:
  - Source Graph and Session v1 are real.
  - Queue / commit semantics and action history are real.
  - `Jam`, `Log`, `Source`, and `Capture` shells are real.
  - TR-909, MC-202, W-30, and Scene Brain slices exist behind the Jam workflow.
- Read the active roadmap phase from live docs:
  - `docs/README.md`
  - `docs/execution_roadmap.md`
  - `docs/phase_definition_of_done.md`

## Source Of Truth

Read these before structural changes:

1. `docs/prd_v1.md`
2. `docs/execution_roadmap.md`
3. `docs/specs/technology_stack_spec.md`
4. `docs/specs/rust_engineering_guidelines.md`
5. `docs/specs/source_graph_spec.md`
6. `docs/specs/source_timing_intelligence_spec.md`
7. `docs/specs/session_file_spec.md`
8. `docs/specs/action_lexicon_spec.md`
9. `docs/specs/audio_core_spec.md`
10. `docs/specs/validation_benchmark_spec.md`
11. `docs/specs/fixture_corpus_spec.md`
12. `docs/specs/audio_qa_workflow_spec.md`
13. `docs/specs/tui_screen_spec.md`
14. `docs/specs/ghost_api_spec.md`
15. `docs/specs/preset_style_spec.md`

Strategic context:

- `plan/riotbox_masterplan.md`
- `plan/riotbox_liam_howlett_feral_addendum.md`

Agent-facing drift guardrails:

- `docs/reviews/riotbox_drift_guardrails_2026-05-10.md`

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
- When fuller harnesses land, tighten this rule to the spec's stronger release gates.

## Rust Guidance

- Keep core types explicit and boring.
- Keep tests close to the modules they validate.
- Avoid unnecessary dependencies during early model stabilization.
- Treat roughly 500 lines per Rust file, including tests and bin helpers, as a soft review/context budget.
- Treat any `.rs` file over that budget as a refactor candidate.
- Never split files mechanically just to satisfy line count.
- Split only when resulting modules have clearer semantic responsibility, lower review cost, and lower agent context cost.
- Do not hide context cost in giant `tests.rs` files.
- Split large tests by behavior area, fixture family, screen, lane, or helper responsibility only when ownership gets clearer.
- Name split Rust shards after their responsibility, such as `event_loop.rs`, `w30_projection.rs`, or `render_policy_tests.rs`.
- Do not use durable `01_...rs`, `02_...rs` numbering.

## Documentation Rules

- Freeze new technical decisions in `docs/research_decision_log.md`.
- Update the corresponding spec in `docs/specs/` when a contract changes.
- Do not bury important architecture decisions only in code comments.
- Keep important workflow rules in repo docs, not only in MemPalace or Linear.

## Context Hygiene

- Keep normal searches focused on live source and canonical docs.
- Let default `rg` respect `.rgignore`.
- `.rgignore` excludes long Linear archives, raw planning transcripts, generated artifacts, and local audio data.
- Search `docs/archive/linear_issues/` only when ticket history is needed.
- Search ignored archive or audio paths explicitly with `rg --no-ignore "..." <path>`.
- Do not paste large generated manifests, WAV metadata dumps, archive batches, or raw transcript sections unless directly needed.
- Prefer specific files and line ranges over entire long documents.

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

## PR And CI Expectations

- Normal slice work should use a PR.
- Direct push to `main` is only acceptable when the user explicitly asks, the change is very small, the change is repo/workflow-meta, and skipping PR does not hide meaningful review risk.
- Every PR should include `Why This Matters`.
- `Why This Matters` must explain:
  - larger phase or milestone
  - product path or architecture seam unlocked
  - what remains bounded, stubbed, or out of scope
- Non-trivial feature PRs should include `Drift Check`.
- `Drift Check` should cover:
  - new or changed `ActionCommand`
  - queue path
  - commit / side-effect path
  - Session / replay consequence
  - user-visible or observer surface
  - test / QA proof
  - added `JamAppState` state
  - added or changed audio-producing behavior
  - shadow-system risk
- After opening a PR, inspect GitHub Actions / CI explicitly.
- If CI is red and relevant to the slice, fix it on the same branch.
- Treat open PRs and in-flight CI as merge gates, not a reason to pause the main implementation lane.
- If CI is running or already green and the branch is locally clean, continue with the next bounded roadmap-aligned slice.
- Re-check open PRs periodically when no webhook/event mechanism is available.
- Do not emit standalone status-only progress reports when there is no blocker.
- Pair user-facing progress updates with the next concrete action.
- Minimum CI checks:
  - formatter status
  - test status
  - lint status
  - slice-specific workflow required by the repo

## Linear Workflow

- Keep Linear updates human-readable.
- Move issues to `In Progress` when work starts.
- Move issues to `In Review` when the PR is open.
- Move issues to `Done` when the PR is merged.
- Keep Linear priorities explicit:
  - `In Progress` / `In Review` -> `High (2)`
  - honest near-next backlog -> `Medium (3)`
  - distant work -> `Low (4)` or unset
  - archive / repo-ops slices -> usually `Medium (3)` unless urgent
- Keep labels orthogonal to projects:
  - projects answer phase
  - labels answer slice type
- Current base labels:
  - `workflow`
  - `archive`
  - `ux`
  - `benchmark`
  - `review-followup`
- Treat workflow and archive obligations as real work, not optional cleanup.
- When delegation is available and a slice is substantial, prefer two parallel lanes:
  - main implementation lane
  - workflow / ops lane for Linear state, project updates, archive prep, and repo-process obligations
- The main thread still owns correctness, review, PR quality, merge readiness, and final integration.
- Keep a small active backlog in Linear.
- Before closing the current ticket loop, ensure Linear still has:
  - 1 issue in progress or in review
  - 1-5 near-next backlog issues
  - coarse milestone-level placeholders only when honest
- Do not fully decompose distant phases into many detailed tickets before nearer slices land.

## Linear Archive And Deletion

- Treat Linear as the active operations layer, not the long-term archive.
- Archive useful context before deleting a completed Linear issue.
- Archive under `docs/archive/linear_issues/`.
- Do archive work as part of closeout, not as a default separate `Archive ...` ticket.
- Use the repo archive as canonical Git-backed history.
- Keep MemPalace focused on live product docs and specs, not archived Linear ticket files.
- Use one archive file per Linear ticket.
- Do not create grouped ticket archives by default.
- Use month files such as `2026-05.md` only as indexes to per-ticket files.
- Keep archive entries structurally uniform.
- Use `RIOTBOX-123.md` for ticket archive files.
- Use `YYYY-MM.md` for monthly index files.
- Use ISO dates (`YYYY-MM-DD`) for archived ticket timestamps.
- Keep metadata fields in the archive template order.
- Use stable final-status terms:
  - `Done`
  - `Canceled`
  - `Duplicate`
  - `Superseded`
- Preserve at minimum:
  - ticket id and title
  - Linear project
  - milestone or phase
  - final ticket status
  - created date
  - implementation start date when known
  - done, merged, canceled, or deleted date when applicable
  - actual repo feature branch
  - why the ticket existed
  - what shipped
  - PR link
  - merge commit
  - follow-up tickets or bounded open questions
- Preserve when useful:
  - Linear-generated branch name when it differed from the actual repo branch
  - Linear issue URL
  - labels
  - assignee or owner
  - deleted-from-Linear date
  - verification summary
  - decision-log or spec links touched by the ticket
- Delete a Linear issue only after:
  - the PR is merged
  - the issue is done
  - the repo archive entry exists
- Verify archive presence by exact file or metadata only:
  - `test -f docs/archive/linear_issues/RIOTBOX-123.md`
  - `rg --no-ignore -n '^- Ticket: `RIOTBOX-123`' docs/archive/linear_issues`
- Do not use MemPalace as the deletion gate.
- Prefer the archive generator before deletion:
  - `scripts/archive_linear_issue.py --ticket RIOTBOX-123 --pr 99 --why "..." --shipped "..."`
- Prefer the deletion helper:
  - `scripts/linear_issue_delete.sh RIOTBOX-123`
- Prefer the closeout helper for repeated cleanup:
  - `scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99`
- If `--mem-status` is used, keep it bounded with `--mem-status-timeout`; MemPalace is optional and must not block branch or Linear cleanup.
- The archive and closeout helpers default to dry-run. Pass `--execute` only after PR merge, archive handoff, and Linear Done state are confirmed.
- Use token auth for deletion:
  - `LINEAR_API_TOKEN=...`
- Do not rely on pasted browser session cookies as the normal workflow path.

## Branch And Git Hygiene

- Do not revert unrelated user changes.
- Keep commits scoped to one coherent slice where possible.
- After a PR is merged, sync local `main`.
- Delete the merged remote feature branch after the PR is merged and local `main` is synced.
- Prefer deleting the exact PR branch:
  - `git push origin --delete feature/riotbox-123-example`
- Never delete `main`, release/protected branches, branches with open PRs, or intentionally long-lived branches.
- For squash-merged PRs, do not rely only on `git branch --merged`.
- Verify PR merge/closed state or known archive status before bulk branch deletion.
- If doing bulk GitHub branch cleanup, first confirm there are no open PRs and remove only stale non-`main` heads.
- Do not amend commits unless explicitly requested.

## Next-Ticket Heuristic

- Derive the next ticket from:
  - `docs/execution_roadmap.md`
  - `docs/phase_definition_of_done.md`
  - the active feature spec for the area
  - actual current repo state
- Prefer the smallest coherent slice that closes the most immediate product or architecture gap.
- Do not define many future tickets in detail before the current slice lands.
- Validate each next ticket:
  - fits the current phase
  - creates visible product progress or removes a real blocker
  - preserves current architecture
  - is small enough to review as one coherent slice
- Prefer roadmap-aligned product-spine work over new side paths.
- After clean closeout, autonomously start the next-best backlog ticket when:
  - the previous slice is merged or fully closed
  - no unresolved review or CI blocker remains
  - the next ticket satisfies this heuristic
  - the Linear backlog remains within the 1-5 near-next rule

## Commands

Keep this section as a short command shortlist. Use `just --list` and `Justfile` for the full command catalog.

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
just mem-status
just mem-search "replay truth"
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

## MemPalace / Memory Notes

- MemPalace is optional dev memory.
- It is not product core.
- It is not a source of truth.
- Canonical truth lives in `docs/`, `plan/`, Linear, and Git history.
- Use MemPalace to complement `rg`, not replace it.
- Do not store new canonical decisions only in MemPalace.
- If something matters, write it into repo docs or Linear.

Repo-local layout:

- `.mempalace/palace/` stores the persistent Chroma database.
- `.mempalace/cache/` stores model and package cache.
- `.mempalace/results/` stores captured evaluation outputs.
- `.mempalace/corpus/` stores copied project corpus for mining.

Operational path:

- Use `just mem-init` for first setup.
- Use `just mem-status` and `just mem-search "..."` for normal work.
- Use `just mem-repair` for index metadata drift such as missing cosine-distance metadata.
- The wrapper uses rootless Podman with pinned `python:3.14.4-slim` and `mempalace==3.3.4`.
- Normal runtime commands run with container networking disabled.
- Image builds require normal registry/network access.
- The wrapper re-mines when mined repo sources changed.
- The wrapper uses a repo-local lock to prevent concurrent mining.
- Do not hand-edit `.mempalace/corpus/mempalace.yaml`.
- The wrapper syncs selected live repo sources into room-specific folders.
- Active rooms: `specs`, `workflow`, `reviews`, `audio_qa`, `plan`, `decisions`, `code`, `documentation`, and `general`.
- If room structure changes, the wrapper rebuilds the palace index on the next mine/status/search.
- The wrapper rebuilds the MemPalace container image only when compose/container files change.

## Sandbox And Environment Notes

### Audio And Device Probing

- Do not assume a failed audio probe inside the sandbox means the machine audio stack is broken.
- Record whether Linux audio validation came from sandbox or real user session.
- Treat sandbox-only audio failures as inconclusive.

### Agent Sandbox Self-Checks

Run these when Riotbox runs inside `agent-sandbox` and host capability is unclear:

```bash
command -v git
command -v cargo
command -v pkg-config
command -v podman
pkg-config --libs --cflags alsa
test -S "/run/user/$(id -u)/podman/podman.sock" && echo podman-socket-ok
```

Interpretation:

- If `pkg-config --libs --cflags alsa` fails, the sandbox cannot build the current Linux audio path cleanly.
- If `podman` is missing or the Podman socket is unavailable, MemPalace operational commands cannot run from inside the sandbox.
- `just` is convenient but not required; prefer direct script commands if `just` is absent.

Preferred solution:

- Bake needed tooling into the sandbox image.
- Use mounts only for host-specific assets or sockets.

### Arch Host Requirements

Audio build requirements:

- `pkg-config` available in the sandbox.
- ALSA headers and pkg-config data visible in the sandbox.
- `PKG_CONFIG_PATH=/usr/lib/pkgconfig`.

Useful Arch host mounts:

- `/usr/include/alsa` -> `/usr/include/alsa`
- `/usr/lib/pkgconfig` -> `/usr/lib/pkgconfig`
- `/usr/lib/libasound.so` -> `/usr/lib/libasound.so`
- `/usr/lib/libasound.so.2` -> `/usr/lib/libasound.so.2`

MemPalace operational requirements:

- `podman` client available in the sandbox.
- `podman compose` support available in the sandbox.

If using host rootless Podman instead of nested Podman:

- Mount `/run/user/<host-uid>/podman/podman.sock`.
- Expose it at the same path or a known sandbox path.
- Set `CONTAINER_HOST=unix:///run/user/<host-uid>/podman/podman.sock`.

In that setup, `scripts/mempalace.sh` can use the host container runtime without full nested container support.

### Git Push Ergonomics

- Ensure SSH auth is available.
- Ensure GitHub host trust is available.
- A temporary `known_hosts` file is a workaround.
- Better sandbox setup: writable `~/.ssh/known_hosts` or pre-seeded GitHub host keys.

### Host Services

- Use `host.containers.internal` for host-local TCP services.
- Do not assume `localhost` means the host. In the sandbox it is container-local.

## Frozen Stack v1

The stack freeze is documented in `docs/specs/technology_stack_spec.md`.

Use these defaults unless a documented spike disproves them:

- `Rust` for core, runtime-facing state, TUI, and audio path.
- `Python` later for the analysis sidecar.
- `JSON` for early persistence and inspection.
- Planned runtime:
  - `cpal` for audio I/O
  - `tokio` for control-plane async work
  - `ratatui` for terminal UI
- Do not replace Rust with Go for the main core.

## Repo Layout

Important paths:

- `crates/riotbox-core`: shared core models and logic.
- `crates/riotbox-app`: app-level orchestration and Jam state wiring.
- `crates/riotbox-audio`: audio runtime and callback-side work.
- `crates/riotbox-sidecar`: sidecar protocol/client work.
- `docs/`: implementation-facing contracts.
- `plan/`: strategy and historical planning material.
- `python/sidecar`: analysis process path.

## Historical Near-Term Build Order

Follow the live roadmap first. This historical order remains useful when orienting early skeleton work:

1. Stabilize core data models.
2. Add serialization roundtrips for `SourceGraph` and `SessionFile`.
3. Build app-level Jam state wiring.
4. Run bounded spikes:
   - audio latency
   - Rust/Python transport
   - deterministic replay
   - session serialization
5. Move into core skeleton runtime work.

Do not jump to advanced DSP, Ghost `perform`, or export-heavy workflows early.

## When In Doubt

- Prefer the smaller, more explicit model.
- Prefer the contract that preserves replayability.
- Prefer realtime boundaries.
- Prefer docs updates over hidden assumptions.
- Ask one concise clarifying question when user feedback mixes later ideas and immediate implementation requests, unless the intended next action is explicit.
