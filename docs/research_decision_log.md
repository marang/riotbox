# Riotbox Research and Decision Log

Version: 0.1  
Status: Draft  
Audience: whole project

---

## 1. Purpose

This log exists to prevent repeated discussion, hidden assumptions, and undocumented architecture drift.

Use it for:

- bounded research spikes
- architecture decisions
- provider choices
- benchmark interpretations
- explicit rejections of alternatives

Do **not** use it as a diary.

---

## 2. Entry Template

```text
ID:
Date:
Topic:
Phase:
Question:
Options considered:
Decision:
Why:
Evidence:
Consequences:
Follow-up:
Status:
```

---

## 3. Initial Entries

### RBX-001

Date: 2026-04-12  
Topic: language and documentation baseline  
Phase: Phase 0  
Question: what should be considered canonical planning documentation?  
Decision: `docs/` holds implementation-facing specs; `plan/` holds strategy and long-form planning; the active feral addendum is `plan/riotbox_liam_howlett_feral_addendum.md`.  
Why: this keeps strategy, archive history, and implementation contracts separated.  
Consequences: future spec work should land in `docs/`, not in new planning sprawl under `plan/`.  
Status: accepted

### RBX-002

Date: 2026-04-12  
Topic: MVP scope framing  
Phase: Phase 0  
Question: what is the MVP proving?  
Decision: the MVP proves the track-to-instrument path, not full generative autonomy and not DAW completeness.  
Why: this aligns engineering effort with the product spine and protects against scope drift.  
Consequences: Ghost `perform`, advanced export polish, and advanced DSP remain off the early critical path.  
Status: accepted

### RBX-003

Date: 2026-04-12  
Topic: feral mode architecture  
Phase: Phase 0  
Question: how should the feral logic live in the system?  
Decision: feral behavior must be implemented as profile / policy / scoring extensions on top of the core system, not as a second architecture.  
Why: this preserves mergeability, replay consistency, and scope discipline.  
Consequences: new feral work should land in existing modules and specs, not in parallel engines or formats.  
Status: accepted

### RBX-004

Date: 2026-04-12  
Topic: stack freeze v1  
Phase: Phase 0  
Question: which stack decisions need to be frozen before the first implementation slice begins?  
Decision: use `Rust` for the core workspace and runtime-facing model layer, keep `Python` reserved for the later analysis sidecar, target `JSON` for early persisted artifacts, and plan around `cpal`, `tokio`, and `ratatui` for the first runtime-capable stack.  
Why: this is the best fit for realtime control, deterministic state, terminal-native UX, and a later MIR sidecar without forcing premature framework commitments.  
Consequences: the first code slice starts as a Rust workspace, while transport and audio choices get validated by bounded spikes rather than more abstract debate.  
Status: accepted

### RBX-005

Date: 2026-04-12  
Topic: deterministic replay model  
Phase: Core Skeleton  
Question: what should Riotbox treat as replay truth, and how should snapshots relate to action replay?  
Decision: replay truth is the combination of frozen source references, frozen Source Graph references, durable committed action history, and optional snapshots that accelerate restore without replacing the action log. `requested_at` is diagnostic, while commit order and musical commit boundary are replay-relevant.  
Why: replay must not depend on rerunning unstable analysis, re-asking Ghost, or reconstructing captured artifacts from ambient state.  
Consequences: future runtime work should add explicit replay-order metadata, make snapshot anchors more concrete, and preserve musical boundary identity for committed actions.  
Status: accepted

### RBX-006

Date: 2026-04-12  
Topic: CPAL audio direction  
Phase: Core Skeleton  
Question: should Riotbox proceed with `cpal` as the low-level audio I/O entry point for the first runtime-capable audio slice?  
Decision: yes. Use `cpal` as the low-level audio I/O layer and isolate it in `crates/riotbox-audio`, with explicit probing and health metrics kept near the stream layer.  
Why: the library matches Riotbox's callback-oriented audio-core direction and exposes the host/device/config/stream concepts needed for a low-level runtime boundary.  
Evidence: official `cpal` documentation confirms default host/device discovery, supported config enumeration, and output stream construction as the core API surface; the local spike prototype compiles against `cpal` and provides a runnable path for host/device/config probing and callback-gap measurement.  
Consequences: later audio work should build a runtime shell above `cpal` rather than replacing it with a higher-level playback abstraction, and health metrics should be captured from the stream layer from the start.  
Status: accepted

### RBX-007

Date: 2026-04-12  
Topic: Rust-Python sidecar transport  
Phase: Analysis Vertical Slice  
Question: what transport should Riotbox use for the first real Rust-to-Python sidecar integration slice?  
Options considered: newline-delimited JSON over `stdio`, Unix domain sockets, localhost TCP, binary message formats such as MessagePack or Protobuf.  
Decision: use newline-delimited JSON over `stdio` for the first sidecar-facing slice, with explicit request IDs and version fields in messages. Keep future socket-based transports open if concurrency or lifecycle needs outgrow `stdio`.  
Why: this is the smallest debuggable process boundary, keeps transport setup simple while request shapes are still settling, and fits the current goal of bounded request/response analysis without dragging realtime code into sidecar concerns.  
Evidence: the `RIOTBOX-9` spike crate successfully spawns a Python sidecar, completes a `ping` roundtrip, and deserializes a Python-produced stub `SourceGraph` into the existing Rust model.  
Consequences: the next analysis-facing slices should build on a narrow synchronous transport contract first, keep progress streaming optional, and move to sockets only when real workload or lifecycle pressure justifies it. This decision does not freeze `Python` or `NDJSON over stdio` as permanent choices; both should be revisited once the sidecar contract carries more message types, stronger versioning pressure, or external lifecycle needs.  
Status: accepted

### RBX-008

Date: 2026-04-12  
Topic: Rust CI baseline  
Phase: Spec Freeze + Core Model  
Question: what minimum automated checks should Riotbox enforce on the Rust workspace at the current project stage?  
Decision: start with one small GitHub Actions workflow that runs `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings` on pushes to `main` and on pull requests.  
Why: these checks cover formatting drift, broken behavior, and obvious lint-level engineering regressions without prematurely building a large automation surface.  
Evidence: the current workspace already uses `cargo` and a matching `just ci` command locally, so the workflow can mirror existing developer expectations instead of inventing a second build path.  
Consequences: contributors should treat those three commands as the local pre-PR baseline, and future CI growth should add replay, benchmark, or screenshot checks only when those contracts become stable enough to enforce.  
Status: accepted

### RBX-009

Date: 2026-04-12  
Topic: audio runtime shell baseline  
Phase: Core Skeleton  
Question: what is the smallest reusable audio runtime boundary Riotbox should introduce after the completed CPAL spike?  
Decision: add a minimal `AudioRuntimeShell` above `cpal` inside `crates/riotbox-audio`, with explicit lifecycle state, typed output metadata, typed health snapshots, and typed startup errors. Keep the shell limited to stream lifecycle and telemetry for now.  
Why: the project needs a real runtime-facing boundary before scheduler, app-level runtime state, or TUI health surfaces can be added. The smallest useful step is a shell that owns the stream and publishes measurable health without overbuilding the engine.  
Evidence: the new runtime shell compiles cleanly, reuses the existing probe path, and passes unit tests for telemetry accounting, faulted health snapshots, and lifecycle transitions.  
Consequences: future runtime work should build transport, scheduler, and app-facing health state on top of this shell rather than creating a second audio runtime abstraction.  
Status: accepted

### RBX-010

Date: 2026-04-12  
Topic: app-layer runtime health state  
Phase: Core Skeleton  
Question: where should Riotbox represent audio and sidecar runtime health before a full TUI exists?  
Decision: keep runtime-facing health state in `riotbox-app`, not in `riotbox-core`. Reuse typed audio health from `riotbox-audio`, model sidecar availability in the app layer, and derive a Jam-facing runtime summary view there.  
Why: runtime health belongs to orchestration and presentation, not to the stable core domain contracts. Keeping it in the app layer avoids pulling service/runtime concerns into `SourceGraph`, `SessionFile`, or core Jam models too early.  
Evidence: the `RIOTBOX-13` slice extends `JamAppState` with audio and sidecar runtime state, derives a separate runtime summary view, and passes tests covering ready, degraded, and faulted states without changing `riotbox-core`.  
Consequences: future TUI work should consume app-layer runtime summaries, while core contracts stay focused on replay-safe domain state.  
Status: accepted

### RBX-011

Date: 2026-04-12  
Topic: scheduler-facing transport boundary model  
Phase: Core Skeleton  
Question: how should Riotbox represent quantized commit timing before the full scheduler exists?  
Decision: add an explicit `TransportClockState` and `CommitBoundaryState` in `riotbox-core`, and let the action queue commit against that boundary state instead of a bare enum alone. Return stable per-boundary commit order from the queue for scheduler-facing use.  
Why: replay and scheduler work need a concrete musical boundary model instead of relying on hidden timing assumptions or incidental queue vector order. The queue should know which musical window it is committing against, not just that it saw a generic `Bar` or `Phrase`.  
Evidence: the `RIOTBOX-14` slice adds transport and boundary types, queue commits against explicit boundary state, and tests stable commit ordering and boundary identity without changing persistence yet.  
Consequences: future scheduler and replay work should build on this explicit boundary model, and persistence can decide later how much of that runtime boundary metadata becomes durable session history.  
Status: accepted

### RBX-012

Date: 2026-04-12  
Topic: app-layer Jam runtime orchestration  
Phase: Core Skeleton  
Question: where should Riotbox first combine transport clock updates, queue commits, and session-facing Jam refresh logic before the full scheduler and TUI exist?  
Decision: add the first runtime orchestration methods in `riotbox-app`, not `riotbox-core`. `JamAppState` should own transport clock updates, commit queued actions against explicit `CommitBoundaryState`, mirror committed actions into the session action log in stable order, and reseed fresh queue IDs after persisted session history.  
Why: this is orchestration work across runtime state, queue semantics, and presentation refresh, not a new core contract. Keeping it in the app layer avoids pulling scheduler/runtime glue into the core model while still making the Jam shell testable.  
Evidence: the `RIOTBOX-17` slice adds app-level transport and commit methods, covers transport refresh and stable commit propagation with tests, and keeps `riotbox-core` limited to reusable queue primitives plus ID reseeding support.  
Consequences: future scheduler/TUI work should build on `JamAppState` orchestration entry points, while persistence and replay continue to rely on the explicit session action log rather than queue internals alone.  
Status: accepted

### RBX-013

Date: 2026-04-12  
Topic: MemPalace as Riotbox dev-memory tooling  
Phase: Core Skeleton  
Question: should Riotbox adopt MemPalace now as a standard internal project-memory and agent-assist retrieval tool?  
Decision: do not make MemPalace a required default workflow dependency yet, but treat it as a validated optional dev-memory tool using rootless Podman with pinned `Python 3.12` and repo-local persistent storage.  
Why: the direct host trial failed on the current machine baseline (`Python 3.14`), but the real rootless Podman evaluation completed successfully against Riotbox data. The remaining uncertainty is not basic operability; it is whether the retrieval value justifies adding another maintained tool beside repo docs and Linear. For Riotbox, an external dev-memory helper is only worth standardizing if setup is boring and it clearly improves real retrieval tasks.  
Evidence: upstream documentation shows active progress and honest correction of earlier overstated claims, but also ongoing backend and stability work; the host trial installed `mempalace 3.1.0` successfully yet failed during runtime import through the `chromadb` / `pydantic.v1` path on Python 3.14. A real rootless Podman trial with pinned `python:3.12-slim` completed `init`, `mine`, `status`, and multiple Riotbox searches successfully, producing a persistent palace under `.mempalace/`.  
Consequences: Riotbox should continue treating repo docs, the decision log, and Linear as the canonical memory layer. MemPalace is now credible enough to keep as an optional retrieval helper, but it should stay outside product core and should not become a second hidden source of truth. Broader workflow adoption should depend on a comparative bakeoff against `rg` plus repo docs plus Linear.  
Status: accepted

### RBX-014

Date: 2026-04-12  
Topic: MemPalace versus `rg` role split  
Phase: Core Skeleton  
Question: after a broader bakeoff, how should Riotbox position MemPalace relative to the existing `rg`-plus-docs workflow?  
Decision: keep MemPalace as an optional semantic project-memory tool, and keep `rg` as the primary exact lookup tool. MemPalace complements `rg`; it does not replace it.  
Why: the broader bakeoff showed that MemPalace is stronger for question-shaped architecture and planning retrieval across documents, while `rg` remains dramatically faster and better for exact code/symbol navigation.  
Evidence: an eight-task comparison against an expanded Riotbox corpus found strong MemPalace results for questions such as Rust-core rationale, replay truth, and feral-profile semantics, but weaker performance on exact implementation lookup like the Jam runtime slice. Query timing also remained much higher for MemPalace than for `rg`.  
Consequences: Riotbox can justify keeping MemPalace available as an optional retrieval layer for long-horizon project memory, but day-to-day code navigation should continue to rely on `rg` first. Any broader default adoption should focus on workflow polish rather than pretending the tools serve the same job.  
Status: accepted

### RBX-015

Date: 2026-04-12  
Topic: MemPalace operational path  
Phase: Core Skeleton  
Question: if Riotbox keeps MemPalace available, what is the supported local operating path?  
Decision: use a repo-local wrapper around rootless Podman Compose with a pinned `python:3.12` image, repo-local state under `.mempalace/`, and automatic re-mining when `docs/`, `plan/`, `crates/`, or `AGENTS.md` changed. Expose it through `scripts/mempalace.sh` and `just` commands.  
Why: this removes the manual container incantation, hides Python-version concerns, keeps state persistent outside ephemeral containers, and makes the optional tool boring enough to use without pretending it is canonical product infrastructure.  
Evidence: the earlier MemPalace evaluation already proved the rootless Podman path works on Riotbox data; the wrapper and compose setup turn that validated path into a repeatable repo-local command surface.  
Consequences: contributors can use MemPalace through stable project commands, while the tool remains optional and subordinate to repo docs, Linear, Git history, and `rg` for exact lookup.  
Status: accepted

### RBX-016

Date: 2026-04-12  
Topic: first analysis ingest slice shape  
Phase: Analysis Vertical Slice  
Question: what is the smallest real analysis-facing path Riotbox should add after the transport spike and core skeleton groundwork?  
Decision: add one app-facing `analyze_source_file` ingest path that sends a source file path through the existing stdio sidecar boundary, receives a `SourceGraph`, persists both graph and session JSON, and returns a ready `JamAppState`. Keep the current Python sidecar implementation deliberately bounded to file-based stub analysis rather than full decode quality.  
Why: the project needs a real path from source file to persisted graph without waiting for full MIR quality or reopening the transport contract. This proves the integration seam through app layer, sidecar transport, persistence, and Jam state assembly in one bounded slice.  
Evidence: `riotbox-sidecar` now supports a real file-path request in addition to the older transport stub request, `riotbox-app` can ingest a real source file through that sidecar path and persist JSON artifacts, and tests cover both the happy path and a missing-file failure path.  
Consequences: the next analysis work should improve the actual analysis quality behind the same ingest path rather than inventing a second graph-loading flow. Persistence and Jam assembly can now assume a real sidecar-produced graph path exists, even though the analysis content is still intentionally simple.  
Status: accepted

### RBX-017

Date: 2026-04-12  
Topic: decoded-source analysis baseline  
Phase: Analysis Vertical Slice  
Question: what is the smallest useful improvement behind the new ingest seam before Riotbox takes on real MIR complexity?  
Decision: keep the existing `analyze_source_file` request and `SourceGraph` response shape, but replace the previous file-size heuristic with a decoded WAV baseline in the Python sidecar. Derive source duration, sample rate, channel count, simple energy summaries, and a duration-fit timing estimate from the decoded audio itself.  
Why: the ingest seam already exists, so the next valuable step is to make the sidecar return graph content grounded in actual decoded source data without reopening transport or overcommitting to premature analysis sophistication.  
Evidence: the sidecar now decodes real PCM WAV input via Python stdlib `wave`, the Rust sidecar and app tests use real generated WAV fixtures instead of arbitrary bytes, the happy-path graph reflects decoded metadata, and unsupported files surface a stable explicit failure path.  
Consequences: future analysis work should continue to improve timing, sectioning, and candidate quality behind this same ingest path. Riotbox now has a bounded decoded-source baseline rather than a pure transport stub, but it remains intentionally simple and WAV-focused for now.  
Status: accepted

### RBX-018

Date: 2026-04-12  
Topic: first TUI-facing Jam shell boundary  
Phase: Analysis Vertical Slice  
Question: once analyzed session state exists, what is the smallest real UI slice Riotbox should add without leaking presentation concerns into the core contracts?  
Decision: add the first terminal UI shell entirely in `riotbox-app` using `ratatui` and `crossterm`, with one file-driven binary that either ingests a source file or loads an existing session/graph pair and renders the current `JamViewModel` plus runtime health. Keep `riotbox-core` presentation-free and test the render path with a non-interactive `TestBackend`.  
Why: the project now needs a user-facing Jam surface to make the current ingest and runtime work visible, but the core contracts should remain reusable and not turn into widget models or terminal-specific abstractions.  
Evidence: `riotbox-app` now has a real `riotbox-app` binary, a minimal Jam shell renderer, argument parsing for ingest/load flows, a render test against `ratatui::backend::TestBackend`, and a smoke-launched TTY shell that exits cleanly.  
Consequences: later UI work should deepen this same app-side shell instead of inventing a separate presentation path. The next TUI slices can add richer panels, keybindings, and screenshots while reusing the current `JamAppState` seam.  
Status: accepted

### RBX-019

Date: 2026-04-13  
Topic: first navigable Jam shell interaction model  
Phase: Jam-first Playable Slice  
Question: after the first shell exists, what is the smallest next interaction layer that makes it meaningfully usable without pretending Riotbox already has deep live controls?  
Decision: keep the shell file-driven and app-side, but add a tiny safe keybinding set: `q`/`Esc` to quit, `?` or `h` to toggle a help overlay, and `r` to refresh the shell by reloading or re-ingesting from the current launch mode. Add clearer source context and section visibility directly in the main Jam surface, and store a normalized terminal baseline under `docs/screenshots/`.  
Why: the shell now needs to answer “what am I hearing” and “what can I do next” more clearly, but the project still is not ready for full performance editing or transport control. Small real interactions and stronger source context improve usability without destabilizing the core contracts.  
Evidence: `riotbox-app` now has shell state for launch mode, help visibility, and status messages; the renderer shows source metadata, analysis confidence, and section summaries; tests cover richer render content and key handling; a real TTY smoke pass exercised help, refresh, and quit; and a stable terminal baseline is stored in the repo.  
Consequences: later TUI work should build on this same shell-state seam for richer keybindings, source trust surfaces, and screenshot updates. Live musical controls remain intentionally out of scope until the shell and app-side runtime become more mature.  
Status: accepted

### RBX-020

Date: 2026-04-13  
Topic: Jam shell trust and action cue framing  
Phase: Jam-first Playable Slice  
Question: after the first live-safe shell interactions exist, what is the next smallest UI improvement that makes the Jam shell feel more like an instrument surface instead of a status dashboard?  
Decision: keep the shell app-side and non-destructive, but reframe the main surface around trust and action imminence. Replace the generic top-row status framing with `Now`, `Next`, `Trust`, and `Lanes`, strengthen the header around current scene and next queued move, and expose recent committed actions and source trust more explicitly. Keep deep live editing and transport control out of scope.  
Why: Riotbox now has a real Jam shell, but the shell still needs to answer “what is happening now,” “what lands next,” and “how much should I trust this analysis” at a glance. Those cues are more important to musical use than generic runtime inventory panels.  
Evidence: `riotbox-app` now renders the shell around trust and action cues, tests validate the richer snapshot semantics, a real TTY smoke pass exercised the updated shell, and a new normalized baseline artifact captures the reviewable result in `docs/screenshots/jam_shell_trust_action_baseline.txt`.  
Consequences: the next Jam shell slices should keep deepening this same path with safe musical cues, pending/commit clarity, and better source context rather than opening a second editor path. Explicit live mutation controls remain intentionally bounded until the shell/runtime seam is stronger.  
Status: accepted

---

## 4. Mandatory Research Topics

The following topics require explicit entries before related implementation scales:

- audio backend and latency baseline
- sidecar transport choice
- deterministic replay model
- analysis provider baseline
- benchmark threshold policy
- Ghost budget and safety policy

---

## 5. Decision Hygiene

Every major decision should record:

- what problem it solved
- what alternative was rejected
- what evidence supported it
- what follow-up work it created

If that is not written down, the decision is not stable enough to rely on.
