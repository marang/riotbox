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

### RBX-021

Date: 2026-04-13  
Topic: first live-safe Jam action entry should stay inside the existing queue and transport seam  
Phase: Jam-first Playable Slice  
Question: now that the Jam shell is readable and safe, how should Riotbox introduce its first user-triggered musical actions without inventing a second interaction or execution path?  
Decision: add a small set of bounded Jam keybindings that enqueue real Action Lexicon entries into the existing `ActionQueue`, advance them through the current transport-boundary model, and keep undo as an app-side action-log operation. Do not add a parallel shell-local mutation executor or device-specific shortcut path.  
Why: the current gap is not more passive UI, but the absence of first-class user action entry. The smallest slice that moves Riotbox toward “instrument” status is to let the Jam shell queue a few real actions and visibly commit them on beat/bar/phrase boundaries. Using the existing queue and transport model preserves replayability and keeps the product on one interaction spine.  
Evidence: `riotbox-app` now exposes first live-safe Jam actions for scene mutation, TR-909 fill, and phrase capture, advances transport on a small app-side tick, commits queued actions through the existing commit-boundary flow, and supports one recent undo path. Tests cover the new queueing, boundary commit, undo, and shell keybinding behavior.  
Consequences: later Jam interaction work should deepen the same queue/transport seam for capture, device MVPs, and stronger pending/commit visibility. Full device execution semantics still remain out of scope for this slice.  
Status: accepted

### RBX-022

Date: 2026-04-13  
Topic: the first Jam capture workflow should materialize real capture records on commit  
Phase: Jam-first Playable Slice  
Question: after live-safe Jam actions exist, what is the smallest capture-oriented slice that makes capture feel like a real workflow instead of only another queued command label?  
Decision: when a committed capture action lands, create a real `CaptureRef`, update the W-30 lane's `last_capture`, and expose the newest capture summary directly in the Jam shell. Do this within the current session/action/view seam instead of adding a separate capture runtime or a full Capture screen.  
Why: Phase E still requires a first capture path. That requirement is not satisfied by merely being able to queue a capture action. The user needs to see that a committed capture produced reusable session state with a target and provenance. Materializing capture records at commit time gives Riotbox its first real capture loop while keeping the slice narrow.  
Evidence: committed capture actions in `riotbox-app` now append `CaptureRef` records with source-origin refs and W-30 targeting, update `last_capture`, and feed a new capture summary in `JamViewModel`. Tests cover capture materialization and the Jam shell now renders a dedicated capture panel.  
Consequences: later Capture-screen work and W-30 MVP work should build on these same session capture records rather than inventing a second capture inventory. Deep sample editing and resample routing remain out of scope for this slice.  
Status: accepted

### RBX-023

Date: 2026-04-13  
Topic: TR-909 MVP should start with explicit lane-state side effects before audible drum generation  
Phase: TR-909 MVP  
Question: after the Jam shell can queue and commit safe actions, what is the smallest TR-909 slice that moves the first device MVP forward without pretending the audio engine already supports real drum reinforcement?  
Decision: let committed TR-909 actions update explicit lane state for fill arming, last fill bar, pattern reference, and reinforcement mode, and surface those cues in the Jam shell. Keep the slice inside the current session/action/view seam and defer actual audible drum synthesis to a later audio-facing step.  
Why: Riotbox needs a real device seam before it can support believable TR-909 audio work. The first honest increment is to make `TR-909 fill` and `TR-909 reinforce` actions produce replayable device state that the shell can show and later audio work can consume. That preserves continuity from Phase E into Phase F instead of jumping directly from UI hints to untracked audio behavior.  
Evidence: `riotbox-core` and `riotbox-app` now track TR-909 fill and reinforcement state explicitly, committed TR-909 actions mutate that state at transport boundaries, the Jam shell surfaces the resulting cues, and tests cover the new side effects plus keybinding entry.  
Consequences: later TR-909 slices should consume the same lane state for audible pattern generation and drum reinforcement rather than bypassing it. This slice does not yet satisfy full TR-909 MVP exit criteria because audible reinforcement still remains out of scope.  
Status: accepted

### RBX-024

Date: 2026-04-13
Topic: the next TUI slice should add a Log screen instead of inventing a second action-trust surface
Phase: Jam-first Playable Slice
Question: after Jam has real queued actions, capture materialization, and the first TR-909 device cues, what is the smallest next UI slice that improves user trust without opening a parallel editor path?
Decision: add the first `Log` screen directly inside the existing shell, with explicit screen switching between `Jam` and `Log`, and render queued, committed, rejected, and undone actions from the current session and queue state. Keep the slice read-only and avoid introducing a second log model or a separate inspector runtime.
Why: the TUI spec already prioritizes `Log` immediately after `Jam`, and the current product gap is not a lack of more device controls but a lack of visible action trust. Now that Riotbox has real actions and side effects, users need a dedicated trust surface that answers what just changed, what is queued, and why outcomes differed.
Evidence: `riotbox-app` now has shell-level screen switching, a first Log screen using existing action/session/runtime state, tests covering screen switching and log rendering, and a normalized baseline artifact in `docs/screenshots/jam_log_screen_baseline.txt`.
Consequences: later TUI work should keep `Jam` as the performance surface and deepen `Log` as the trust/history surface instead of adding parallel inspector screens for the same information. Filtering, search, and Ghost-specific log detail remain out of scope for this slice.
Status: accepted

### RBX-025

Date: 2026-04-13
Topic: the next TUI slice after Log should add the Source screen inside the same shell spine
Phase: Jam-first Playable Slice
Question: once `Jam` and `Log` both exist, what is the smallest next UI slice that improves analysis trust without opening a separate source-inspector runtime?
Decision: add the first `Source` screen directly inside the existing shell, extend screen switching so `Jam`, `Log`, and `Source` are all reachable in one TUI spine, and render source identity, timing confidence, sections, candidate summaries, provenance, and source-graph warnings from the existing `SourceGraph`.
Why: the TUI spec puts `Source` immediately after `Log`, and Riotbox already has enough decoded-source structure that users should be able to inspect it in one dedicated place. The next honest improvement is better visibility into analysis-derived structure, not a second inspector toolchain.
Evidence: `riotbox-app` now renders a dedicated `Source` screen, tests cover the new screen and screen switching, and a normalized baseline artifact exists at `docs/screenshots/source_screen_baseline.txt`.
Consequences: later source-oriented work should deepen this same screen for richer structure trust and candidate inspection instead of creating a separate source-debug UI. Deep graph editing and Ghost-specific diagnostics remain out of scope for this slice.
Status: accepted

---

Topic: the next TUI slice after Source should add the Capture screen inside the same shell spine  
Phase: Jam-first Playable Slice  
Question: once `Jam`, `Log`, and `Source` all exist, what is the smallest next UI slice that makes capture feel like a first-class musical workflow without inventing a separate capture runtime?  
Decision: add the first `Capture` screen directly inside the existing shell, extend screen switching so `Jam`, `Log`, `Source`, and `Capture` all remain in one TUI spine, and render readiness, recent captures, provenance, pending capture cues, and routing context from the existing session, queue, and `JamViewModel` state.  
Why: the TUI spec calls out `Capture` as a core screen, and Riotbox already materializes real capture records plus queued capture actions. The next honest improvement is to make that workflow legible in the same shell, not to create a second capture inventory or a deeper W-30 editor before the capture path is visible.  
Evidence: `riotbox-app` now renders a dedicated `Capture` screen, tests cover the new screen and screen switching, and a normalized baseline artifact exists at `docs/screenshots/capture_screen_baseline.txt`.  
Consequences: later capture-oriented work should deepen this same screen for promotion, pinning, and reuse cues instead of opening a separate capture browser. Deep sample editing and full W-30 pad workflow remain intentionally out of scope for this slice.  
Status: accepted

---

Topic: the first capture-promotion flow should bind existing captures instead of pretending capture already equals promotion  
Phase: Jam-first Playable Slice  
Question: now that the `Capture` screen exists, what is the smallest next slice that makes captured material reusable in-flow without inventing a separate W-30 subsystem?  
Decision: keep capture and promotion as distinct steps. Committed capture actions create capture records that may remain unassigned, while `promote.capture_to_pad` updates an existing capture's target inside the current action/session/view seam and surfaces the promotion result directly in the `Capture` screen.  
Why: the PRD requires captured material to be reusable without leaving flow, but collapsing capture and promotion into one side effect makes the workflow semantically muddy and hides the real promotion seam. Distinguishing them keeps the architecture honest while still staying bounded inside the current shell and session model.  
Evidence: `riotbox-app` now queues `promote.capture_to_pad` from the shell, applies promotion as a side effect on an existing `CaptureRef`, updates the action result summary, tracks promoted vs unassigned capture counts in `JamViewModel`, and refreshes the `Capture` screen plus baseline artifact to show the new state. Tests cover both unassigned capture materialization and later promotion.  
Consequences: later W-30 work should build on this same `capture -> promote` path for pinning, pad reuse, and promotion history instead of reintroducing implicit auto-routing. Deep resample routing and full pad editing remain intentionally out of scope for now.  
Status: accepted

---

Topic: the next capture slice should add persisted pinned-capture recall instead of a second capture browser  
Phase: Jam-first Playable Slice  
Question: after the first promotion path exists, what is the smallest next slice that makes capture recall more intentional without opening a new capture-management subsystem?  
Decision: add persisted pin metadata directly to `CaptureRef`, expose pinned capture counts and IDs through `JamViewModel`, and let the shell toggle pin state for the latest capture through a small app-layer interaction. Keep this as explicit session metadata rather than creating a separate favorite store or a new action family.  
Why: the TUI spec explicitly calls out favorite or pinned captures, and the MVP needs meaningful capture recall. A persisted pin flag is enough to make the capture workflow feel more deliberate without disturbing the existing capture/promotion seam or inventing a second browser path.  
Evidence: the session model now stores pin state on captures, `riotbox-app` can toggle pin state for the latest capture, the `Capture` screen shows a dedicated pinned section, and tests cover both the pin toggle and the updated screen rendering.  
Consequences: later capture work should deepen this same persisted pinning path for favorites and reuse flows instead of duplicating capture metadata elsewhere. Deep tagging or folder-like capture management remains out of scope for now.  
Status: accepted

---

Topic: pending TR-909 fill intent should be derived from the queue, not persisted as committed lane state  
Phase: Jam-first Playable Slice  
Question: how should Riotbox surface a queued TR-909 fill without creating save/reload drift between pending intent and committed runtime state?  
Decision: keep the pending TR-909 fill as queue-only intent. `queue_tr909_fill` should not mutate persisted lane state ahead of commit, and the `JamViewModel` should derive its “fill armed” visibility from the pending queue instead of from `session.runtime_state`.  
Why: the TUI contract requires users to distinguish queued from committed state, and the prior implementation could save committed-looking lane state even though the fill action still existed only in memory. Deriving the indicator from the queue preserves the visible cue without lying about what has already happened.  
Evidence: `queue_tr909_fill` no longer flips `fill_armed_next_bar` in the session, `JamViewModel` now computes the armed indicator from pending `tr909.fill_next` actions, and app tests now verify that saving with a queued fill does not persist committed lane state across reload.  
Consequences: later work that needs durable pending-action restoration must either persist the queue explicitly or keep all pending-only cues derived from pending action data rather than from runtime state.  
Status: accepted

---

Topic: ingest should default to embedded graph storage and only write external graph files when explicitly requested  
Phase: Jam-first Playable Slice  
Question: how should the source-file ingest seam store its `SourceGraph` during MVP so it stays aligned with the current session-file contract without losing the ability to write explicit external graph files later?  
Decision: make embedded graph storage the default ingest path. When no explicit graph path is requested, the ingest flow writes the `SourceGraph` into the session as an embedded graph reference. External graph files remain supported only when the caller passes an explicit graph path.  
Why: the current session-file spec says MVP should prefer embedded graphs unless graph size becomes a real problem. Defaulting ingest to external files created extra file coupling and diverged from the current contract without any demonstrated need.  
Evidence: the ingest entry point now accepts an optional graph path, CLI parsing defaults `--source` ingest to no external graph file, tests cover both explicit external graph output and the default embedded-graph path, and save/load behavior continues to work in both modes.  
Consequences: later work can still add explicit external-graph workflows without changing the default MVP contract. Multi-source session questions remain separate and are tracked under the follow-up contract-alignment ticket.  
Status: accepted

---

Topic: Riotbox MVP should make the current single-source assumption explicit instead of silently collapsing plural session refs  
Phase: Jam-first Playable Slice  
Question: should the app start supporting multiple source refs now, or should MVP explicitly freeze to one active source and one matching graph ref until multi-source work is intentionally designed?  
Decision: freeze MVP explicitly to a single active source and a single matching source-graph reference. Keep the schema plural for forward compatibility, but make the spec explicit and reject invalid multi-source sessions in the app/runtime instead of silently loading only the first graph.  
Why: the current app behavior is single-source already. Leaving the plural core shape unqualified made the contract look broader than the runtime really was, which would create silent drift and make later multi-source work harder to reason about.  
Evidence: the app load path now validates the single-source MVP constraint, rejects sessions with multiple source refs or mismatched source/graph IDs, and the session-file spec now calls the MVP restriction out explicitly while preserving the plural schema shape for future migration.  
Consequences: later multi-source work will need an explicit active-source selector and updated app/runtime contracts instead of relying on the current single-source assumption.  
Status: accepted

---

Topic: terminal UI should consume runtime pulses instead of owning transport advancement  
Phase: Jam-first Playable Slice  
Question: how should Riotbox remove musical timing authority from the terminal redraw loop before the full audio scheduler exists?  
Decision: introduce a small app-runtime pulse source outside the TUI event loop and make the app own elapsed-time transport advancement from those pulses. The terminal loop should only render snapshots, dispatch key intents, and consume already-timed runtime signals.  
Why: the periodic codebase review identified a real architecture problem: the shell poll tick was both advancing transport and deciding boundary commits. That made musical timing depend on redraw cadence, which conflicts with the audio-core contract and would force a rewrite once audio or a real scheduler becomes authoritative.  
Evidence: `riotbox-app` now has a dedicated runtime pulse source, the terminal binary no longer computes beat deltas, and `JamAppState` advances transport from elapsed pulse timestamps through an explicit driver state. Tests cover runtime-anchor setup and elapsed-time transport progression while preserving the existing queue and commit-boundary behavior.  
Consequences: later scheduler or audio-runtime work can replace the current pulse source with a stronger timing authority without reopening the TUI contract. The shell stays bounded to rendering and intent dispatch, while the app/runtime seam becomes the place where transport time enters the product.  
Status: accepted

---

Topic: the first TR-909 MVP increment should be one bounded slam control on the existing action seam  
Phase: TR-909 MVP  
Question: what is the smallest real TR-909 control Riotbox should add now that the review-driven cleanup queue is closed and the shell already supports queued lane actions?  
Decision: start the TR-909 MVP with a single `tr909.set_slam` control queued through the existing `ActionQueue` and committed on the current transport-boundary seam. Keep it as a bounded toggle-like live control rather than inventing a separate device subsystem or pretending full drum takeover behavior already exists.  
Why: the roadmap says TR-909 comes first among device MVPs, but the next slice still needs to stay small and reviewable. A bounded slam control adds a real device-facing interaction and visible lane change without reopening transport, persistence, or TUI architecture.  
Evidence: the shell now exposes a dedicated `s` keypath for `tr909.set_slam`, the app queues it as a normal action, the committed side effects update both the TR-909 lane state and the macro intensity, and tests cover queueing, duplicate-pending protection, and committed slam state.  
Consequences: later TR-909 work should deepen the same queue-and-commit seam for reinforcement and takeover behavior rather than adding shortcut execution paths. Full audible drum takeover and richer pattern semantics remain out of scope for this first device-facing increment.  
Status: accepted

---

Topic: the next TR-909 MVP increment should add explicit takeover and release actions on the existing seam  
Phase: TR-909 MVP  
Question: how should Riotbox add controlled TR-909 lane takeover without inventing a second execution path or hiding state transitions behind UI-only toggles?  
Decision: add explicit `tr909.takeover` and `tr909.release` actions, queue them through the existing action seam, and commit them on phrase boundaries. Represent the committed state in TR-909 lane state and expose the pending target separately in the Jam view so the shell can show queued-versus-committed takeover clearly.  
Why: the milestone requires controlled 909 takeover, but the current shell only had fill, reinforce, and slam controls. A takeover/release pair is the next smallest real device-facing increment that keeps replay, queueing, and view state aligned.  
Evidence: the core action vocabulary now includes dedicated takeover and release commands, the app queues them with duplicate-pending protection, committed side effects update lane takeover state on phrase boundaries, and the Jam shell shows both committed takeover state and any queued takeover/release change. Tests cover queueing guards plus takeover and release commits.  
Consequences: later TR-909 work should deepen takeover semantics behind the same commands, for example richer pattern adoption or audio-facing render seams, instead of introducing a separate lane-control system.  
Status: accepted

---

Topic: the first audio-facing TR-909 render seam should be a derived runtime contract, not a second lane-control path  
Phase: TR-909 MVP  
Question: after TR-909 takeover and release exist on the normal action seam, what is the smallest next slice that prepares audible reinforcement work without prematurely building a drum engine or duplicating lane logic inside the audio crate?  
Decision: add a small `riotbox-audio` TR-909 render contract and derive it from committed session lane state plus transport and mixer context inside `riotbox-app`. Expose the derived render mode and routing in the shell, but keep actual drum synthesis out of scope for this slice.  
Why: Phase 3 is only really done once reinforcement becomes audible, but the next honest increment is not full drum generation. Riotbox first needs one explicit audio-facing contract that later render code can consume without bypassing the queue, replay, or committed lane state.  
Evidence: `riotbox-audio` now defines a dedicated TR-909 render state, `riotbox-app` derives that state from the committed session/runtime model on refresh, tests cover idle, reinforce, takeover, and release projections, and the Jam shell shows the current render seam summary.  
Consequences: later audible TR-909 work should consume this render contract and extend it if necessary instead of re-deriving drum state from ad-hoc UI cues or introducing a second device-state system.  
Status: accepted

---

Topic: the first audible TR-909 reinforcement slice should stay inside the existing render seam and audio runtime shell  
Phase: TR-909 MVP  
Question: once Riotbox has an explicit TR-909 render contract, what is the smallest next step that makes reinforcement honestly audible without pretending a full drum-machine engine already exists?  
Decision: drive a bounded callback-side TR-909 reinforcement renderer directly from the existing render seam, and start that audio path from the Jam app without introducing a second device-control system or a separate drum runtime. Keep the sound generation intentionally simple and replay-aligned: support, reinforce, and takeover should become audibly distinct, but full pattern adoption and richer device semantics stay out of scope.  
Why: Phase 3 requires audible reinforcement, but jumping from a render contract straight to a full TR-909 engine would be too large and too architecture-risky. The honest next move is to make the existing seam produce audible results while preserving the queue, transport, and committed lane-state path.  
Evidence: `riotbox-audio` now renders bounded TR-909 reinforcement audio from the render seam in the audio callback, `riotbox-app` starts the audio runtime and keeps it updated from the current committed render state, and tests cover silent idle behavior, audible support-mode output, and zero drum-bus silence.  
Consequences: later TR-909 slices should deepen the same audible render path with better profiles, pattern adoption, and regression fixtures rather than replacing it with a parallel drum subsystem.  
Status: accepted

---

Topic: early TR-909 render profiles should be typed and derived from source plus committed lane state  
Phase: TR-909 MVP  
Question: now that TR-909 reinforcement is audibly real, how should Riotbox make support and takeover sound semantically different without reintroducing stringly callback logic or a second device-control path?  
Decision: keep render-profile choice inside the existing TR-909 render contract and make it typed. Derive source-support profiles from the current source section at the app layer, derive takeover profiles from the committed TR-909 lane state, and let the audio callback consume those typed profiles to vary density, gain, pitch, and decay.  
Why: the next honest step after first audible reinforcement is to deepen musical differentiation, not to invent a parallel drum engine. Typed render profiles preserve the app-to-audio seam, keep the callback free of string parsing, and make profile behavior testable and replay-aligned.  
Evidence: `riotbox-audio` now defines explicit support and takeover render-profile enums, `riotbox-app` derives source-support profiles from source-section context and takeover profiles from committed lane state, and tests cover both the app-side derivation and callback-side audible differences between profiles.  
Consequences: later TR-909 work should extend the same typed render-profile seam with richer pattern adoption and fixture coverage instead of pushing profile semantics back into UI strings or bypassing the current render contract.  
Status: accepted

---

Topic: early TR-909 audible regression coverage should be fixture-backed at both render projection and callback levels  
Phase: TR-909 MVP  
Question: now that Riotbox has audible TR-909 reinforcement plus typed render profiles, what is the smallest verification slice that protects replay-safe behavior without adding new musical logic?  
Decision: add two bounded fixture-backed regression layers. Keep one app-side fixture that checks committed session and source state still project into the expected TR-909 render seam, and one audio-side fixture that checks the callback renderer still produces bounded audible metrics for key render cases.  
Why: the new TR-909 path now spans committed lane state, render projection, and callback output. If that chain drifts silently, Phase 3 audio can become audibly wrong while still compiling. Fixture-backed checks preserve the replay-aligned seam and make later refactors safer without pretending to be full golden-audio approval tests.  
Evidence: `riotbox-app` now loads committed render-projection fixtures for source-support and takeover states, `riotbox-audio` now loads callback render fixtures for idle, source-support, and takeover cases, and both fixture suites run inside the normal Rust test path.  
Consequences: later TR-909 audio work should extend the same fixture-backed regression pattern with richer pattern cases and diagnostics instead of relying only on ad-hoc unit assertions.  
Status: accepted

---

Topic: TR-909 audible render diagnostics should stay read-only and ride on the existing Jam and Log shell seams  
Phase: TR-909 MVP  
Question: once TR-909 reinforcement is audible and fixture-backed, what is the smallest next shell slice that helps humans inspect the render contract without opening a new control path or device-state system?  
Decision: surface concise TR-909 render diagnostics directly in the existing Jam and Log screens. Keep the Jam screen focused on at-a-glance lane cues, and add a compact TR-909 render panel in the Log screen for mode, routing, profile, pattern, mix, and alignment summaries.  
Why: the render seam is now musically meaningful enough that users and reviewers need to see what the audio path believes it is rendering. The next honest step is not more control, but better observability on top of the same committed render contract.  
Evidence: `riotbox-app` now derives richer TR-909 render summaries and warnings from the committed render state, the Jam screen shows concise mode/profile/pattern/mix cues, the Log screen exposes a dedicated render diagnostics panel, and app tests cover both the runtime view derivation and the updated shell snapshots.  
Consequences: later TR-909 work should extend the same read-only diagnostic seam with richer audible metrics or trust cues rather than inventing a separate render-debug model or device inspector outside the current shell.  
Status: accepted

---

Topic: the first TR-909 pattern-adoption step should become typed render state before the audio callback deepens further  
Phase: TR-909 MVP  
Question: once the TR-909 audible seam is observable, what is the next bounded slice that makes the render path more musical without turning `pattern_ref` into callback-side string logic or opening a second drum-engine model?  
Decision: add a typed `pattern adoption` layer to the existing TR-909 render contract. Derive it at the app layer from committed `pattern_ref`, render mode, and current profile context, and let the audio callback vary subdivision, trigger density, accenting, gain, and decay from that typed adoption signal.  
Why: the next honest TR-909 step is to make the current audible seam adopt a bounded pattern shape, not to invent a full device sequencer. A typed adoption layer keeps the render path replay-aligned, keeps string parsing out of the callback, and makes the new musical behavior testable and diagnosable in the shell.  
Evidence: `riotbox-audio` now defines an explicit `Tr909PatternAdoption` enum, `riotbox-app` derives it from committed render context, the callback changes its audible behavior from that adoption layer, fixtures cover both the app-side projection and audio-side regression cases, and the Jam/Log shell diagnostics surface the adopted pattern name.  
Consequences: later TR-909 work should extend the same typed pattern-adoption seam with phrase-aware variation and richer fixtures instead of bypassing it with direct callback heuristics or a separate device-state graph.  
Status: accepted

---

Topic: TR-909 phrase-aware variation and release behavior should extend the existing render seam instead of creating a second phrase engine  
Phase: TR-909 MVP  
Question: after typed pattern adoption exists, what is the next bounded slice that makes the TR-909 lane feel more phrase-aware without adding a second timing model, release engine, or device-control path?  
Decision: add a typed `phrase variation` layer to the existing TR-909 render contract. Derive it at the app layer from committed transport phrase context, current mode/profile context, and explicit release-pattern cues, then let the audio callback vary subdivision, trigger activity, pitch, gain, and decay from that typed phrase variation.  
Why: the next honest TR-909 step is to deepen the existing audible seam so it responds to phrase context and release state, not to invent a second sequencer or phrase-specific runtime model. A typed phrase-variation layer keeps the behavior replay-aligned, preserves the current queue/commit seam, and makes phrase behavior fixture-testable at both render-projection and callback levels.  
Evidence: `riotbox-audio` now defines an explicit `Tr909PhraseVariation` enum, `riotbox-app` derives phrase variation from transport phrase state and release-pattern context, the callback changes audible behavior from that variation layer, fixtures now cover release-tail cases, and the Jam/Log shell diagnostics surface the current phrase variation label.  
Consequences: later TR-909 work should continue extending the same typed render seam with richer musical behavior rather than bypassing it with direct callback heuristics, UI-only phrase modes, or a separate device-state graph.  
Status: accepted

---

Topic: the first MC-202 MVP control should be a committed role toggle on the existing queue seam
Phase: MC-202 MVP
Question: after the current TR-909 lane is stabilized, what is the smallest honest MC-202 entry slice that creates real device progress without opening a second control path or pretending the follower generator already exists?
Decision: start MC-202 with a bounded `mc202.set_role` action that toggles between `follower` and `leader` on the existing `ActionQueue` and `NextPhrase` commit seam. Surface the pending target in the Jam shell, update committed lane state plus a simple phrase reference on commit, and keep generation itself out of scope.
Why: Riotbox needs a real first MC-202 control, but the follower/answer generation path is not ready yet. A committed role toggle uses the existing replay-aligned action seam, makes the lane visible and queueable in the shell, and avoids inventing a parallel device-control path just to enter the milestone.
Evidence: `riotbox-app` now queues `mc202.set_role` as a phrase-boundary action, the commit path updates `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, `riotbox-core` exposes pending MC-202 role intent in `JamViewModel`, and shell tests cover both the keybinding and the pending-role cue.
Consequences: later MC-202 work should build follower/answer generation and live parameter control on top of the same committed role seam rather than bypassing it with direct UI-only state or a second lane model.
Status: accepted

---

Topic: the first MC-202 follower generator should stay phrase-quantized on the existing role seam
Phase: MC-202 MVP
Question: after committed role control exists, what is the smallest honest next slice that creates a usable MC-202 follower-line path without pretending a full synth engine, phrase editor, or answer generator already exists?
Decision: add a bounded `mc202.generate_follower` action on the existing `ActionQueue` and `NextPhrase` commit seam. Surface pending follower generation in the Jam shell, commit it into `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, and keep deeper answer logic and live parameter editing out of scope.
Why: Phase 4 needs a real follower-line path, not just another role toggle. Reusing the current phrase-boundary seam keeps generation replay-safe and visible, while committed lane-state updates make the MC-202 lane feel real without inventing a second phrase engine or UI-only device model.
Evidence: `riotbox-app` now queues `mc202.generate_follower` as a phrase-boundary action, the commit path writes a follower-oriented phrase reference plus touch intensity into session state, `riotbox-core` exposes pending follower generation in `JamViewModel`, and shell tests cover both the new keybinding and the pending-generation cue.
Consequences: later MC-202 work should extend the same committed phrase seam with answer generation and parameter control rather than bypassing it with direct shell state, callback-only heuristics, or a separate MC-202 runtime graph.
Status: accepted

---

Topic: the first deeper MC-202 shell slice should improve lane diagnostics instead of opening a synth inspector
Phase: MC-202 MVP
Question: after the follower-generation action exists, what is the next bounded shell slice that makes the MC-202 lane more legible without inventing a dedicated device pane or second diagnostic surface?
Decision: deepen the existing `Jam` and `Log` screens with clearer MC-202 lane diagnostics. Keep the slice read-only on top of committed lane state, pending-action intent, and the existing action log, and avoid any new device-specific editor or inspector route.
Why: once follower generation exists, users need to see what the MC-202 lane believes it is doing. The shell should answer that directly in the normal operator surfaces instead of forcing a second panel or hidden debug path.
Evidence: `riotbox-app` now shows richer MC-202 lane summaries in `Jam`, adds a dedicated `MC-202 Lane` diagnostics panel in `Log`, and keeps the diagnostics grounded in existing `JamViewModel` fields plus the action log. The footer also now advertises the follower-generation key so visible controls match the actual shell behavior.
Consequences: later MC-202 work should keep deepening these same operator surfaces instead of opening a second synth inspector or a parallel device-debug workflow.
Status: accepted

---

Topic: the first W-30 MVP slice should reuse the capture and promotion seam for live recall
Phase: W-30 MVP
Question: after capture, promotion, and pinning exist, what is the smallest honest W-30 entry slice that creates a real live-recall cue without inventing a second sample browser, pad editor, or playback-control surface?
Decision: start W-30 with a bounded live-recall cue on the existing `w30.swap_bank` action seam. Queue recall against the latest pinned promoted capture first, fall back to the latest promoted capture, commit it on `NextBar`, and update lane focus plus the capture reference on commit.
Why: Riotbox already has enough capture and promotion state to expose a truthful first W-30 cue. Reusing that seam keeps the entry slice replay-safe, visible in the Jam shell, and grounded in the current session model instead of opening a parallel W-30 control path too early.
Evidence: `riotbox-app` now queues `w30.swap_bank` as a live-recall cue, prefers pinned promoted captures for recall targeting, updates `w30.active_bank`, `w30.focused_pad`, and `w30.last_capture` on commit, and surfaces the pending recall in the Jam and Capture shell views. `riotbox-core` now carries W-30 focused-pad and pending-recall state in `JamViewModel`, and app/UI tests cover both the queue path and the committed side effects.
Consequences: later W-30 work should build audible audition, recall variations, and deeper pad handling on top of the same capture/promotion and committed recall seam instead of bypassing it with direct shell-only state or a separate W-30 browser model.
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
