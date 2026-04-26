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

Topic: the first MC-202 answer generator should reuse the existing phrase seam instead of opening a second phrase engine
Phase: MC-202 MVP
Question: after committed follower generation exists, what is the next bounded MC-202 slice that deepens phrase interplay without pretending a full answer editor, callback-side sequencer, or hidden phrase graph already exists?
Decision: add a bounded `mc202.generate_answer` action on the existing `ActionQueue` and `NextPhrase` commit seam. Surface pending answer generation in the Jam shell, commit it into `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, and keep deeper phrase editing and live synth control out of scope.
Why: the current MC-202 lane already has a replay-safe phrase seam and committed lane-state model. Extending that seam with answer generation creates a real next musical response without inventing a second phrase engine or a UI-only device model.
Evidence: `riotbox-app` now queues `mc202.generate_answer` on `NextPhrase`, commits it into answer-oriented lane state plus touch intensity, exposes the pending answer cue in the shell, and covers the path with fixture-backed regression tests alongside the existing role and follower cases.
Consequences: later MC-202 work should keep extending the same committed phrase seam, including richer answer behavior and parameter controls, instead of bypassing it with direct shell state, callback-only heuristics, or a separate MC-202 runtime graph.
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

Topic: promoted-material audition should stay on the existing W-30 pad cue seam instead of opening a second browser
Phase: W-30 MVP
Question: after live recall exists, what is the next bounded W-30 slice that makes promoted material feel more like an instrument without inventing a second sample browser, pad editor, or callback-only preview path?
Decision: add one explicit `w30.audition_promoted` action on the existing `ActionQueue` and `NextBar` seam. Queue it against the latest promoted W-30 capture, block conflicting pending W-30 pad cues, surface the pending audition target in the shared shell summary, and commit it into the same lane focus seam plus a bounded `w30_grit` bump.
Why: the repo already has a real capture, promotion, and live-recall seam. Extending that seam with promoted-material audition deepens the W-30 lane musically while keeping the slice replay-safe and visible in the current shell instead of inventing a second W-30 model.
Evidence: `riotbox-app` now queues `w30.audition_promoted` on `NextBar`, commits it through the same W-30 side-effect path that updates bank, pad focus, and last-capture state, adds a bounded grit bump, and exposes the pending audition cue in the Jam and Capture shell views. Tests cover cue conflict blocking, committed side effects, and the shell-visible pending cue.
Consequences: later W-30 work should keep building audible preview and deeper pad behavior on top of the same committed cue seam instead of bypassing it with shell-only flags or a separate preview browser.
Status: accepted

---

Topic: live drum-bus control should stay on the existing mixer seam instead of opening a mixer page
Phase: Jam-First Playable Slice
Question: how should Riotbox close the current usability gap where the TR-909 render seam can be technically running but still silent because the drum bus is at zero?
Decision: add one bounded live drum-bus level control directly in the Jam shell and keep it on the existing persisted `mixer_state.drum_level` seam. Update the Jam and Log shell summaries from the same render-mix projection and avoid opening a second mixer page or a parallel callback-side control model.
Why: the repo already has the right seam: persisted mixer state, app-derived render summaries, and a running audio path. A small live control makes the current render seam audibly testable by ear without widening scope into a full mixer surface.
Evidence: `riotbox-app` now adjusts `session.runtime_state.mixer_state.drum_level` live from the shell, refreshes the same `tr909_render_mix_summary`, and keeps the render warning active when the drum bus reaches zero. Tests cover mixer-state adjustment and the new shell keybindings.
Consequences: later mixer work should deepen the same session/runtime seam with clearer controls and diagnostics rather than bypassing it with a second page or callback-only volume state.
Status: accepted

---

Topic: TR-909 scene-lock variation should reuse the existing takeover seam instead of adding a second editor path
Phase: TR-909 MVP
Question: after takeover, release, pattern adoption, and phrase variation exist, what is the next bounded TR-909 control that deepens the lane musically without opening a second TR-909 editor or phrase engine?
Decision: add one explicit `tr909.scene_lock` action on the existing `ActionQueue` and `NextPhrase` commit seam. Commit it into the same takeover lane state already used by the audio-facing render projection by setting `takeover_enabled`, `takeover_profile`, `pattern_ref`, and `reinforcement_mode`, and surface the pending profile in the Jam shell instead of creating a second TR-909 control model.
Why: the codebase already had a typed `scene_lock` render profile and fixture coverage, but no honest committed control could reach it. A bounded scene-lock action deepens the current TR-909 MVP through the same replay-safe seam that already drives takeover and release, while avoiding a hidden render-only mode or a separate device editor path.
Evidence: `riotbox-core` now treats `tr909.scene_lock` as part of the canonical action vocabulary, `riotbox-app` queues it on `NextPhrase` with the same pending-guard used by takeover and release, committed side effects drive `scene_lock_takeover` lane state and render projection, and Jam-shell key handling plus tests cover the new pending-profile and committed scene-lock path.
Consequences: later TR-909 work should keep extending the same committed takeover seam, including richer scene-lock behavior, instead of bypassing it with callback-only toggles or a separate TR-909 variation editor.
Status: accepted

---

Topic: W-30 diagnostics should deepen the existing Capture and Log screens instead of opening a second control surface
Phase: W-30 MVP
Question: after live recall and promoted-material audition exist, what is the smallest next slice that makes the W-30 lane legible for operators without inventing a separate W-30 page, browser, or hidden preview state?
Decision: surface bounded W-30 diagnostics in the existing `Capture` and `Log` screens. Use the current `JamViewModel`, session capture inventory, and committed action log to show pending cue kind, focused bank/pad, latest promoted capture, last lane capture, and the most recent committed W-30 cue outcome.
Why: the repo already has a truthful W-30 seam for recall and audition, but the shell still hides too much of that state inside generic action history. The next honest step is to make the current seam explain itself in-place, not to open a second W-30 control surface or a preview-only browser.
Evidence: `riotbox-app` now adds a dedicated `W-30 Lane` diagnostics panel to `Log`, deepens `Capture -> Routing / Promotion` with explicit pending cue and promoted-target context, and covers the new shell cues with snapshot-style tests for queued and committed W-30 states.
Consequences: later W-30 work should continue extending these same screens and the existing committed cue seam for audible preview and deeper pad behavior instead of bypassing them with separate shell-only panels or hidden preview state.
Status: accepted

---

Topic: W-30 MVP should gain shared replay-safe regression fixtures before deeper audio-facing preview work
Phase: W-30 MVP
Question: after live recall, promoted-material audition, and shell diagnostics exist, what is the smallest next slice that hardens the current W-30 lane before it grows into an audio-facing preview seam?
Decision: add one shared W-30 regression fixture corpus in `riotbox-app` and reuse it in both committed-state and shell-visible tests. Cover the shipped `live recall` and `promoted audition` cues, assert committed lane state plus result summaries at the app layer, and assert Capture/Log shell output from the same fixture data.
Why: TR-909 and MC-202 already use fixture-backed regressions to keep the current seam replay-safe while the device lane grows. W-30 needed the same verification net before deeper preview or pad behavior could be added honestly.
Evidence: `riotbox-app` now has `w30_regression.json`, fixture-backed committed-state regressions in `jam_app`, and fixture-backed shell regressions in `ui` for both recall and promoted-audition paths.
Consequences: later W-30 work should extend the same fixture corpus when preview render state or deeper pad behavior lands instead of relying only on ad hoc unit tests or manual shell checks.
Status: accepted

---

Topic: W-30 audio-facing preview should start as one typed render seam instead of direct sample playback
Phase: W-30 MVP
Question: after live recall, promoted-material audition, diagnostics, and replay-safe regression coverage exist, what is the smallest next slice that prepares the W-30 lane for later audible preview without opening a second device model or pretending full sample playback is already solved?
Decision: add one typed `W30PreviewRenderState` in `riotbox-audio`, derive it only from the existing committed session and action seam in `riotbox-app`, and mirror it into the audio runtime alongside the existing TR-909 render state. Surface the preview mode, profile, target, and mix summary in the current shell/runtime summaries, but stop short of real W-30 sample playback in this slice.
Why: Riotbox needed an honest audio-facing seam for the W-30 lane before pads become audible or internally resampled. The smallest correct move is to make the preview state explicit and callback-reachable using the same replay-safe lane, capture, and action log state that already drives recall and audition, instead of inventing a hidden preview-only model or jumping straight to sample rendering.
Evidence: `riotbox-audio` now has a dedicated `w30` render-state module and shared runtime storage, `AudioRuntimeShell` updates that preview state on the same path as TR-909 render updates, and `riotbox-app` derives typed preview mode/profile/routing from committed W-30 state and exposes it through `JamRuntimeView` plus shell regressions.
Consequences: later W-30 work should attach audible preview rendering, pad playback, and resample taps to this same typed preview seam rather than bypassing it with ad hoc callback state or a separate W-30 playback path.
Status: accepted

---

Topic: W-30 audible preview should deepen the typed preview seam and keep fresh ingest sessions audibly reachable
Phase: W-30 MVP
Question: once the typed W-30 preview seam exists, what is the smallest honest slice that makes it audibly testable without bypassing the committed render model or opening a second playback path?
Decision: keep W-30 preview audio on the same `W30PreviewRenderState` seam and render it inside the existing audio callback alongside TR-909, using one bounded lo-fi preview synth that responds to preview mode, source profile, grit, transport, and music-bus level. Also open the music bus to a modest default level for fresh ingest sessions so the new audible seam is reachable in normal app launches.
Why: Riotbox already had the right preview contract and callback plumbing, but the W-30 lane was still silent even when preview state was active. The smallest correct move is to make that seam audibly real in place, not to add a separate W-30 player, hidden callback-only state, or a one-off shell preview path. Giving fresh ingest sessions a nonzero music bus keeps the new seam practically testable instead of leaving it gated behind an implicit zero-level default.
Evidence: `riotbox-audio` now mixes a dedicated W-30 preview renderer in the existing output callback, covers live recall, promoted audition, zero-music silence, and stopped-preview audibility with runtime tests, and `riotbox-app` initializes fresh ingest sessions with a nonzero `mixer_state.music_level` so committed W-30 preview work can actually be heard.
Consequences: later W-30 work should keep extending this same seam with richer preview profiles, real pad playback, and deeper diagnostics, and should treat music-bus defaults and controls as part of the same mixer path instead of inventing a second W-30-only gain model.
Status: accepted

---

Topic: Playable W-30 pad hits should use a committed trigger action on the existing preview seam
Phase: W-30 MVP
Question: after live recall, promoted audition, and audible preview exist, what is the smallest next slice that makes the W-30 lane feel playable without inventing a second playback path or bypassing replay-safe action state?
Decision: add one explicit `w30.trigger_pad` action quantized to `next_beat`, keep it on the same committed W-30 lane and action log seam as recall and audition, and carry the trigger through the existing `W30PreviewRenderState` using a monotonic trigger revision plus trigger velocity so the audio callback can retrigger the current pad accent in place.
Why: Riotbox needed a first playable W-30 control, but the existing architecture already had the right lane focus, capture selection, and preview callback seam. The smallest honest move is to commit a trigger action and let the current preview renderer retrigger from that committed state, instead of adding a hidden one-shot player, direct shell-to-audio trigger, or a second W-30 engine.
Evidence: `riotbox-core` now exposes `w30.trigger_pad`, `riotbox-app` queues it on `next_beat`, carries pending trigger cues in the Jam view, commits it into lane state plus result summaries, and derives trigger revision and velocity in `W30PreviewRenderState`. `riotbox-audio` now retriggers the current W-30 preview accent when that revision changes, with app, UI, and runtime regressions covering the new path.
Consequences: later W-30 pad features should keep using this committed trigger seam for replay-safe one-shot behavior, and should extend the existing preview render state instead of creating direct callback-only trigger plumbing.
Status: accepted

---

Topic: W-30 audible preview diagnostics should stay inside the existing Jam and Log shell surfaces
Phase: W-30 MVP
Question: once W-30 preview is audible and the first playable trigger exists, what is the smallest next slice that makes that preview state legible without opening a separate W-30 control page?
Decision: keep W-30 audible preview diagnostics inside the existing `Jam` and `Log` shell surfaces, and derive compact mode, target, mix, and trigger cues from `JamRuntimeView` plus the committed lane state instead of creating a dedicated W-30 browser or a second preview-only panel hierarchy.
Why: the runtime seam already exposes the information needed to understand what the audible preview path is doing. The smallest honest move is to summarize that existing seam where operators already look, not to add a second surface that would drift from the committed queue and render model.
Evidence: `JamRuntimeView` now carries an explicit W-30 trigger summary on top of the existing preview mode, target, and mix summaries; the Jam shell now surfaces those cues inline in the main lane overview; the Log screen deepens the `W-30 Lane` panel with the same audible preview state; and the normalized review artifact at `docs/screenshots/w30_audible_preview_baseline.txt` records the result.
Consequences: later W-30 diagnostics should keep extending these same shell summaries and the current typed preview seam instead of splitting preview interpretation across a second W-30-only surface or a callback-only debug overlay.
Status: accepted

---

Topic: W-30 internal resample taps should extend explicit capture lineage instead of inventing a second capture system
Phase: W-30 MVP
Question: after audible preview, pad triggering, and shell diagnostics exist, what is the smallest next slice that prepares internal W-30 resample taps without pretending the full resample lab already exists?
Decision: add one typed `W30ResampleTapState` on top of the current W-30 runtime seam and derive it only from explicit `CaptureRef` lineage metadata plus the committed W-30 lane focus. Extend `CaptureRef` with `lineage_capture_refs` and `resample_generation_depth`, default new captures to generation zero, and keep the first shell proof point inside the existing capture-oriented shell summaries instead of opening a separate resample page.
Why: Riotbox already has real capture records, promotion, audible preview, and a typed W-30 runtime seam. The smallest honest preparation step for internal resample taps is to make capture-to-capture lineage explicit and mirror one tap-ready state through that same seam, not to open a second capture inventory, a hidden preview-only resample model, or a full resample-lab UI.
Evidence: `riotbox-core` now persists explicit capture lineage metadata, `riotbox-audio` now carries a typed `W30ResampleTapState`, `riotbox-app` derives that tap-ready state from the current lane capture and runtime mix context, and the shell baseline at `docs/screenshots/w30_resample_tap_baseline.txt` records the first compact tap cue on the existing Capture surface.
Consequences: later W-30 resample actions and internal bus taps should populate the same lineage fields, deepen the same tap state, and keep using the existing shell surfaces instead of bypassing session capture history or creating a second W-30 resample runtime.
Status: accepted

---

Topic: the first W-30 internal resample action should stay on the committed capture lineage seam
Phase: W-30 MVP
Question: once internal resample lineage metadata and tap-ready state exist, what is the smallest next slice that makes W-30 internal resampling feel real without opening a second capture inventory, pad editor, or callback-only resample trigger?
Decision: add one explicit `promote.resample` action on the existing `ActionQueue` and `NextPhrase` seam for the W-30 lane. Queue it only against the current committed W-30 lane capture, block duplicate pending W-30 resample actions, and materialize the committed result as a new `CaptureRef` with explicit lineage and incremented `resample_generation_depth`.
Why: the repo already has the right ingredients for internal resampling: committed W-30 lane focus, explicit capture lineage metadata, and a typed resample tap summary. The smallest honest move is to create one replay-safe committed resample action on top of that same seam, not to invent a second capture browser, a hidden callback-only resample path, or a full W-30 lab before the lineage model is exercised for real.
Evidence: `riotbox-app` now queues `promote.resample` against the current `w30.last_capture`, commits it on `NextPhrase`, creates a new `CaptureRef` with cloned source-origin refs plus extended lineage and generation depth, updates the W-30 lane to point at the new capture, and surfaces the pending cue in the current shell flow. App and shell tests cover the queue path, duplicate blocking, committed lineage materialization, and the capture-screen cue.
Consequences: later W-30 internal buses, pad-bank behavior, and deeper resample tooling should continue extending this committed lineage seam instead of bypassing it with hidden resample state or direct callback-only mutation.
Status: accepted

---

Topic: W-30 resample lineage diagnostics should stay in the existing Capture and Log shell surfaces
Phase: W-30 MVP
Question: once the first committed W-30 internal resample action exists, what is the smallest next slice that keeps lineage provenance legible without opening a second W-30 diagnostics page or resample-only browser?
Decision: deepen the existing `Capture` and `Log` surfaces with compact lineage diagnostics derived from the committed lane capture and the typed resample-tap seam. Surface pending resample intent explicitly, show compact generation and lineage counts in the W-30 log lane, and keep fuller lineage-chain context in the capture routing panel instead of creating a separate resample screen.
Why: the repo already has truthful resample state, but after `RIOTBOX-65` operators still had to infer too much lineage from generic capture summaries. The smallest honest move is to summarize the existing committed seam where users already look, not to split W-30 provenance across a second diagnostics hierarchy that could drift from the queue and runtime state.
Evidence: `riotbox-app` now shows pending `promote.resample` intent in the W-30 shell cue, renders compact tap and lineage summaries in `Capture -> Routing / Promotion`, compresses generation and lineage counts into the `Log -> W-30 Lane` panel, and covers the new wording with capture and log shell regressions.
Consequences: later W-30 lab work should keep extending these same shell surfaces and the current committed lineage seam instead of moving provenance into a second W-30-only browser or a callback-only debug path.
Status: accepted

---

Topic: W-30 capture resolution should follow committed lane focus before pad-bank stepping lands
Phase: W-30 MVP
Question: after the periodic codebase review flagged capture-driven W-30 helpers, what is the smallest correction that keeps later bank-step and trigger work honest on the current preview seam?
Decision: when explicit `w30.active_bank` and `w30.focused_pad` exist, resolve W-30 recall, audition, trigger, and internal resample actions from the latest committed capture assigned to that focused pad. Only fall back to the older latest-capture or latest-promoted heuristics when no explicit lane focus exists.
Why: bank-step work becomes partly cosmetic if committed focus can move without changing the capture chosen by recall, audition, trigger, or resample actions. The smallest honest fix is to make the existing helpers respect committed lane focus first, not to invent a second W-30 selection model or defer the inconsistency until after more pad-bank controls land.
Evidence: `riotbox-app` now resolves focused W-30 captures before queueing recall, audition, trigger, and resample actions, and the regression tests explicitly cover focused-bank capture selection instead of the older latest-promoted-only behavior.
Consequences: later pad-bank stepping should update committed lane focus and rely on the same resolver rather than choosing captures through separate shell-only or queue-only heuristics.
Status: accepted

---

Topic: W-30 preview mode should be explicit committed lane state rather than action-log reconstruction
Phase: W-30 MVP
Question: after the periodic codebase review flagged preview-mode reconstruction from `action_log`, what is the smallest correction that keeps the audible preview seam deterministic and replay-safe?
Decision: persist explicit W-30 preview intent in `runtime_state.lane_state.w30.preview_mode`, update it from committed W-30 preview-facing actions, and build the runtime preview seam from that explicit state. Keep a one-time legacy backfill from committed W-30 preview actions only when loading older sessions that do not yet carry the explicit field.
Why: the preview seam should depend on one committed source of truth, not on whichever W-30 action happens to be latest in the historical log. Making preview intent explicit keeps later pad-bank stepping and lane-focus work honest, while the one-time backfill preserves compatibility for older saved sessions without leaving the runtime builder tied to history forever.
Evidence: `riotbox-core` now persists explicit W-30 preview intent in lane state, `riotbox-app` updates it during committed W-30 side effects, and regression tests cover both legacy backfill and the rule that explicit lane state overrides stale action history.
Consequences: future W-30 controls should update `preview_mode` through committed lane state instead of teaching the runtime builder about more action-log patterns. Replay and restore semantics stay explicit even as more W-30 controls land.
Status: accepted

---

Topic: Pending W-30 resample cues should enter the shell through the core Jam view model
Phase: W-30 MVP
Question: after the periodic codebase review flagged a shell-side queue scan for pending W-30 resample intent, what is the smallest correction that restores the presentation boundary without creating a second W-30 summary path?
Decision: extend `JamViewModel.lanes` with explicit pending W-30 resample cue data and have the shell derive its cue label from that core presentation contract instead of scanning `ActionQueue` directly.
Why: the shell already receives pending MC-202, TR-909, recall, audition, and trigger summaries through the core Jam view model. Leaving pending W-30 resample intent as a direct queue scan would keep one small but real boundary leak alive, making later shell work easier to drift into ad hoc queue inspection.
Evidence: `riotbox-core` now surfaces `w30_pending_resample_capture_id` in `LaneSummaryView`, the Jam-view regression fixture covers that new field, and `riotbox-app` no longer scans `ActionQueue` directly for the W-30 resample cue label.
Consequences: future shell summaries should keep extending the core Jam view contract rather than introducing new queue reads inside `ui.rs`. Queue-level state remains modeled centrally before it is rendered.
Status: accepted

---

Topic: first bounded W-30 pad-bank stepping should use an explicit committed focus-step action on the preview seam
Phase: W-30 MVP
Question: once committed W-30 lane focus and preview mode are explicit, what is the smallest next slice that lets operators step across promoted pads without inventing a second shell cursor, pad editor, or preview-only state machine?
Decision: add one explicit `w30.step_focus` action on the existing `ActionQueue` and `NextBeat` seam for the W-30 lane. Resolve its target from the next promoted W-30 pad after the current committed lane focus, block it against other pending W-30 pad cues, and update committed W-30 lane focus plus the existing preview seam when it lands.
Why: stepping should be a real committed musical control, not a shell-only cursor move disguised as recall. A dedicated `w30.step_focus` action keeps pending cues, recent action summaries, and replay history honest while still staying bounded to the current W-30 preview model instead of opening a full pad-bank editor early.
Evidence: `riotbox-core` now exposes `w30.step_focus` explicitly, the Jam view surfaces pending focus-step targets, `riotbox-app` queues it on `NextBeat` from actual promoted W-30 pads, the shell binds it directly, and the app/UI regressions cover both pending and committed focus-step behavior.
Consequences: later W-30 bank-grid work should continue extending this explicit committed focus seam rather than smuggling pad stepping through recall semantics or a separate shell-only focus cursor.
Status: accepted

---

Topic: W-30 live recall should stop overloading the bank-swap action name before bank-manager controls land
Phase: W-30 MVP
Question: once committed focus-step behavior exists, what is the smallest honest cleanup that keeps later bank-manager work from inheriting misleading W-30 action semantics?
Decision: split the existing live-recall behavior onto an explicit `w30.live_recall` action command and reserve `w30.swap_bank` for future real bank-manager movement. Keep the queue target resolution, `NextBar` quantization, committed preview behavior, and focused-pad side effects otherwise unchanged.
Why: the repo already moved pad stepping onto its own committed `w30.step_focus` seam. Leaving live recall on `w30.swap_bank` would make action history misleading and would force the first actual bank-manager slice to either reuse a dishonest command name or create another workaround around the same seam.
Evidence: `riotbox-core` now exposes `w30.live_recall`, the Jam view uses it for pending recall summaries, `riotbox-app` queues recall with that explicit command while preserving the existing recall targeting logic and committed side effects, and the shell baselines plus queue/commit tests were updated to show `w30.live_recall` instead of `w30.swap_bank` for recall cues.
Consequences: later W-30 bank-manager work can now use `w30.swap_bank` for real bank changes without rewriting old recall history again. The current live-recall seam stays replay-safe, but its action log and shell labels are now honest about what the slice actually does.
Status: accepted

---

Topic: first bounded W-30 bank-manager control should swap committed focus across promoted banks without opening a second bank editor
Phase: W-30 MVP
Question: now that live recall has its own explicit action seam, what is the smallest next slice that turns `w30.swap_bank` into a real bank-manager control while staying on the existing preview and commit boundaries?
Decision: use `w30.swap_bank` as a `NextBar` control that rotates to the next promoted W-30 bank, preserves the current focused pad when that pad exists in the target bank, falls back to the first promoted pad in that bank otherwise, and commits the same W-30 preview-facing lane updates through the existing focused-pad seam.
Why: the current W-30 MVP already has explicit committed focus, preview mode, and live recall semantics. A first real bank swap should therefore be a bounded movement across existing promoted banks, not a new shell-only bank cursor or a second bank-manager state machine. Reusing the committed focus seam keeps action history honest and lets later bank-grid work refine the same path instead of replacing it.
Evidence: `riotbox-app` now resolves `w30.swap_bank` from actual promoted W-30 targets, queues it on `NextBar`, blocks it against other pending W-30 pad cues, updates lane focus plus the last capture on commit, and distinguishes pending bank cues from recall cues in the shell. `riotbox-core` now carries a dedicated `w30_pending_bank_swap_target`, and queue/commit plus shell regressions cover both the pending and committed bank-swap behavior.
Consequences: later W-30 bank-manager work should keep extending this explicit committed bank-swap seam rather than inventing a separate bank navigation surface. The current slice stays bounded to promoted-bank rotation only; full bank-grid editing, empty-bank travel, and deeper pad-forge controls remain out of scope.
Status: accepted

---

Topic: first bounded W-30 pad-forge control should apply one explicit damage profile on the current preview seam
Phase: W-30 MVP
Question: once committed W-30 trigger, recall, bank-swap, and internal resample behavior exist, what is the smallest honest first step toward pad-forge behavior without introducing per-pad forge state or a second W-30 editor?
Decision: implement `w30.apply_damage_profile` as one bounded `NextBar` control on the current focused W-30 capture seam. It targets the current preview-facing pad capture, reuses the existing committed bank and pad focus path, and raises the existing `w30_grit` macro to one explicit `shred` profile level instead of inventing a full per-pad damage model yet.
Why: the repo already has one replay-safe W-30 preview seam with explicit bank, pad, capture, and grit state flowing into the audio runtime. A first pad-forge move should deepen that seam instead of bypassing it with a hidden forge editor, a callback-only grit toggle, or a prematurely detailed damage-profile schema that the current session model cannot yet persist honestly.
Evidence: `riotbox-app` now queues `w30.apply_damage_profile` on `NextBar`, resolves it from the current W-30 targetable capture, blocks it against other pending W-30 pad cues, preserves the current preview mode while raising committed `w30_grit`, and records an explicit damage-profile result summary on commit. `riotbox-core` now surfaces pending damage-profile targets in `JamViewModel`, and queue, commit, and shell regressions cover both the pending cue and the committed grit update.
Consequences: later W-30 pad-forge work should refine this same committed damage seam instead of introducing a separate forge state machine. The current slice remains intentionally bounded to one `shred` profile and one global grit macro; per-pad forge persistence, multiple named damage profiles, and deeper bank-grid editing remain out of scope.
Status: accepted

---

Topic: W-30 bank-manager and pad-forge follow-ups should deepen the current shell diagnostics instead of opening a second diagnostics surface
Phase: W-30 MVP
Question: once `w30.swap_bank` and `w30.apply_damage_profile` exist on the committed preview seam, how should the shell expose them without regressing the current W-30 capture, lineage, and preview diagnostics?
Decision: keep the slice presentation-only and make the existing `Jam`, `Capture`, and `Log` surfaces carry explicit bank-manager and pad-forge diagnostics. Show the bank-manager state as one compact status next to the current pending cue, show pad-forge state next to the current W-30 mix and capture cues, and compress the Log/Capture wording enough that older lineage and trigger diagnostics stay visible in the same panels.
Why: the repo already has one honest W-30 shell spine. A diagnostics follow-up should deepen that spine instead of adding a separate W-30 debug page or a second forge-specific surface. The main risk in this slice is not missing state, but crowding the fixed terminal layout enough to hide older preview and lineage cues, so the shell wording needs to stay compact and explicit.
Evidence: `riotbox-app` now surfaces explicit bank-manager and pad-forge diagnostics in the `Jam` lane summary, the `Capture -> Routing / Promotion` panel, and the `Log -> W-30 Lane` panel. New shell regressions cover committed bank-swap plus damage-profile state, while existing W-30 shell regressions still pass after the wording compaction. The review artifact at `docs/screenshots/w30_bank_forge_diagnostics_baseline.txt` records the updated shell cues.
Consequences: later W-30 shell work should keep extending the same Jam/Capture/Log surfaces unless the roadmap explicitly calls for a new operator surface. Deeper W-30 forge behavior remains out of scope here; the slice only makes the current committed bank-manager and pad-forge moves legible.
Status: accepted

---

Topic: W-30 bank-manager and pad-forge hardening should extend the shared W-30 regression corpus instead of creating a second fixture path
Phase: W-30 MVP
Question: once `w30.swap_bank` and `w30.apply_damage_profile` have shipped on the committed preview seam, how should the repo harden them without fragmenting the current W-30 regression story?
Decision: extend the existing `w30_regression.json` corpus so the new bank-manager and pad-forge controls use the same fixture-backed committed-state and shell regression path as live recall and promoted audition. Add only the extra fixture metadata needed to express multi-bank setup and initial W-30 preview state, and keep the slice verification-only.
Why: the W-30 MVP already has one honest replay-safe regression seam for committed app state and shell output. Bank swap and damage profile are the same class of committed preview-lane actions, so giving them a separate fixture file or a second one-off test harness would create drift in the repo’s verification model rather than just widening the existing safety net.
Evidence: `riotbox-app` now covers `live_recall`, `promoted_audition`, `swap_bank`, and `apply_damage_profile` from the shared `w30_regression.json` corpus. The same corpus now drives committed-state assertions in `jam_app` and shell-visible assertions in `ui`, including the new bank-manager and pad-forge diagnostics shipped in `RIOTBOX-75`.
Consequences: later W-30 controls on the same committed preview seam should keep extending the shared fixture corpus unless they require genuinely new runtime dimensions. The slice remains intentionally verification-only and does not change shipped W-30 behavior.
Status: accepted

---

Topic: W-30 internal resample taps should become audibly real on the existing audio callback instead of staying diagnostics-only
Phase: W-30 MVP
Question: once the app/runtime seam already derives a typed `w30_resample_tap` state, what is the smallest honest next step that makes it audible without introducing a second W-30 render path?
Decision: route the existing `W30ResampleTapState` through the same realtime callback that already mixes TR-909 support and W-30 preview. Keep the slice bounded to one synthetic internal-capture tap voice that reacts to source profile, lineage depth, generation depth, grit, transport-running state, and music-bus level, and verify it with direct audio callback tests instead of opening a second fixture harness yet.
Why: the repo already had explicit app/runtime diagnostics for internal resample lineage, but the audio runtime still ignored that state. Making the current seam audibly real is the smallest step that turns the typed W-30 resample path into product-visible behavior while preserving the one-callback audio architecture and avoiding a hidden W-30-only render loop.
Evidence: `riotbox-audio` now threads shared resample-tap state into `build_silent_output_stream`, snapshots it in the callback, and mixes a bounded `render_w30_resample_tap_buffer(...)` voice next to the existing TR-909 and W-30 preview paths. Direct runtime tests now cover idle silence, audible lineage-ready taps, and zero-music-bus silence for the new seam, and the full repo verification loop stays green.
Consequences: later W-30 resample-lab work should keep extending this same callback seam instead of creating a parallel internal-resample renderer. Richer profile fixtures, shell diagnostics, and loop-freezer reuse cues remain follow-up work and should build on the same typed runtime state.
Status: accepted

---

Topic: W-30 resample-lab diagnostics should stay in the existing Jam, Capture, and Log shell spine after the audible seam lands
Phase: W-30 MVP
Question: once internal resample taps are audibly real on the callback seam, how should the shell expose that state without opening a second W-30 lab page or regressing older preview, bank-manager, and pad-forge cues?
Decision: keep the slice presentation-only and deepen the existing shell spine with compact resample-lab diagnostics. Jam now shows one `tap` summary next to the current W-30 mix line, Capture now shows source, route, mix, and lineage for the current resample tap, and Log now uses a compressed resample-lab line pair that fits the existing W-30 lane panel width.
Why: the repo already has one honest W-30 operator surface. After `RIOTBOX-77`, the risk is not missing state but leaving the audible resample seam hard to read unless operators inspect generic action history or source lineage by hand. Extending the current Jam/Capture/Log path keeps the shell aligned with the one existing W-30 runtime seam instead of adding another diagnostics surface.
Evidence: `riotbox-app` now renders explicit resample-tap summaries across Jam, Capture, and Log, the shell regressions cover both committed lineage diagnostics and a cross-surface resample-lab snapshot, and the normalized artifact at `docs/screenshots/w30_resample_lab_diagnostics_baseline.txt` records the expected cues.
Consequences: later W-30 resample work should keep extending these same shell surfaces unless the roadmap explicitly calls for a separate operator page. The current slice remains presentation-only and does not change the shipped audio behavior from `RIOTBOX-77`.
Status: accepted

---

Topic: audible W-30 internal resample taps should use the same fixture-backed callback regression pattern as the other shipped audio seams
Phase: W-30 MVP
Question: once the resample-tap callback path is audibly real, how should the repo harden it without inventing a one-off verification style?
Decision: add a dedicated `w30_resample_audio_regression.json` fixture corpus and a callback-level test that evaluates the same active-sample and peak bounds already used for TR-909 and W-30 preview. Keep the slice verification-only and leave the shipped render behavior unchanged.
Why: the repo already has one honest audio-regression pattern for callback-visible behavior. Leaving W-30 internal resample taps on direct one-off tests alone would make the newest audible seam easier to drift than the older TR-909 and W-30 preview paths. Extending the existing fixture shape is the smallest consistent hardening move.
Evidence: `riotbox-audio` now parses `w30_resample_audio_regression.json`, maps fixture rows into `RealtimeW30ResampleTapState`, and verifies idle silence, transport-running lineage-ready taps, stopped-tap audibility, and zero-music-bus silence through the same active-sample and peak assertions used elsewhere.
Consequences: later W-30 audio callback work should keep widening the shared fixture-backed regression net instead of adding seam-specific harnesses. This slice changes no runtime behavior; it only makes future drift on the audible resample seam easier to catch.
Status: accepted

---

Topic: first W-30 loop-freezer reuse should stay on the existing capture seam instead of opening a second reuse editor path
Phase: W-30 MVP
Question: once W-30 preview, bank-manager, damage-profile, and internal-resample seams already exist, what is the smallest honest way to let operators freeze and reuse a loop without inventing a parallel W-30 editor flow?
Decision: add one bounded `w30.loop_freeze` action on the current W-30 capture seam. Queue it on `NextPhrase`, reuse the currently committed W-30 capture target, materialize exactly one new pinned capture on commit, preserve capture lineage through explicit `lineage_capture_refs`, and keep the same W-30 preview/runtime path after commit.
Why: the repo already has one replay-safe W-30 capture and preview seam. A first freezer cue should deepen that seam instead of creating a second “reuse lab” path with separate persistence, routing, or preview rules. The main risk in this slice is lineage drift, not missing UI surface, so the action needs to leave reuse explicit in the same capture model and shell surfaces.
Evidence: `riotbox-core` now exposes `w30.loop_freeze` in the action lexicon and Jam view, while `riotbox-app` queues it on the existing W-30 lane, commits it into a new pinned capture with preserved lineage, and surfaces the pending/committed freeze cues in `Jam`, `Capture`, and `Log`. The shared `w30_regression.json` corpus now covers the committed freeze case for both app-state and shell regressions.
Consequences: later W-30 freezer and reuse work should keep extending the same capture lineage and preview seam unless the roadmap explicitly calls for a fuller editor workflow. Richer reuse browsing, loop editing, and multi-slot freeze management remain follow-up work and stay out of scope here.
Status: accepted

---

Topic: first W-30 slice-pool browse should stay on the current pad-lineage seam instead of opening a second browser model
Phase: W-30 MVP
Question: once loop-freeze reuse already leaves multiple captures on one W-30 pad target, what is the smallest honest next slice that lets operators step through that pool without inventing a separate slice browser, inventory model, or preview-only state?
Decision: add one bounded `w30.browse_slice_pool` action on the existing W-30 lane. Queue it on `NextBeat`, cycle through the captures already assigned to the currently focused W-30 bank/pad, commit it through the same preview-side-effect seam as live recall, and surface only a minimal pending cue in the existing shell.
Why: the repo already has one replay-safe W-30 capture and preview path. After loop-freeze, the immediate need is not a richer browser but a small way to move across the current pad’s committed reuse pool. Reusing the existing lane focus, `last_capture`, and preview-mode seam keeps the slice deterministic and visible without opening a shadow W-30 inventory architecture.
Evidence: `riotbox-core` now exposes `w30.browse_slice_pool` in the action lexicon and Jam view, while `riotbox-app` queues it against the current W-30 target, commits the next capture in that target’s assigned pool into `last_capture`, keeps preview mode on the existing live-recall path, and surfaces the pending browse cue in the shell. Queue and committed-state tests cover the bounded browse behavior.
Consequences: later W-30 slice-pool work should keep extending this same committed pad-lineage seam unless the roadmap explicitly calls for a fuller browse/editor workflow. Richer cross-pad slice browsing, preview profiling, and dedicated diagnostics remain follow-up slices.
Status: accepted

---

Topic: first W-30 slice-pool browse should project a distinct preview profile on the existing live-recall seam
Phase: W-30 MVP
Question: once slice-pool browse is committed on the current pad-lineage seam, what is the smallest honest next step that makes that consequence audible without inventing a second W-30 preview/editor mode?
Decision: keep browse on the existing `W30PreviewRenderMode::LiveRecall` seam and add one typed `W30PreviewSourceProfile::SlicePoolBrowse`. Derive it only from the last committed `w30.browse_slice_pool` action, surface it in the Jam shell as `recall/browse`, and give the audio callback one bounded browse-specific envelope/frequency pattern behind the same preview state.
Why: the current W-30 MVP already has one replay-safe preview seam and one committed pad-lineage model. A first browse consequence should deepen that seam instead of opening a parallel browser-preview architecture with separate persistence or routing rules. The real need is a distinct committed preview consequence, not a richer editor.
Evidence: `riotbox-app` now derives `slice_pool_browse` from committed browse history while keeping preview mode on `live_recall`, fixture-backed app and shell regressions cover the browse case, and `riotbox-audio` now encodes the new profile in shared state plus callback-level audio regressions to keep browse audibility distinct from normal promoted recall.
Consequences: later slice-pool work should continue extending the same committed preview seam unless the roadmap explicitly introduces a fuller browse/editor workflow. Richer pool visualization, cross-pad navigation, and deeper preview shaping remain bounded follow-up slices.
Status: accepted

---

## 4. Mandatory Research Topics

Topic: first Scene Brain slice should project deterministic scene candidates from source sections into existing session state
Phase: Scene Brain
Question: what is the smallest honest first Scene Brain step after W-30 MVP that creates multiple usable scene candidates without opening a second arrangement or scene-graph architecture?
Decision: derive deterministic scene candidate IDs from ordered `SourceGraph.sections`, store them in the existing session `scene_state.scenes`, and normalize `active_scene` plus transport `current_scene` onto that same committed state when no scene is already set. Keep the first slice bounded to candidate projection and the shell visibility that already exists.
Why: the current repo already has one explicit session scene state, one transport scene pointer, and Jam shell visibility for current scene and scene count. A first Scene Brain slice should deepen that spine instead of inventing a second scene graph or separate arrangement inventory before selection and transition semantics even exist.
Evidence: `riotbox-app` now derives scene candidates from analyzed section order during ingest and app-state normalization, and targeted tests prove both empty-session projection and persisted ingest state without changing the current queue or launch architecture.
Consequences: later Scene Brain work should build scene-select, launch, and restore behavior on the same session and transport state. Richer scene graphs, energy management, and transition logic remain follow-up slices.
Status: accepted

---

Topic: first Scene Brain selection should queue one committed `scene.launch` on the existing transport seam
Phase: Scene Brain
Question: once Riotbox already derives deterministic scene candidates, what is the smallest honest next step that lets the operator move to another scene without inventing a second arrangement path, transition model, or editor workflow?
Decision: add one bounded `scene.launch` action that cycles to the next committed scene candidate, queues on `NextBar`, commits through the existing action queue and transport boundary seam, and updates only the current session `active_scene` plus transport `current_scene`. Keep richer scene launch, restore, and transition logic out of scope.
Why: the repo already has one explicit scene list, one transport scene pointer, and one replay-safe queue and commit model. The next honest move is not a richer scene editor but a single committed scene-select control that proves scene changes can stay explicit, logged, and replay-safe on the existing seam.
Evidence: `riotbox-app` now queues `scene.launch` for the next candidate, blocks duplicate pending scene launches, commits it on the current bar boundary, and updates both session scene state and transport scene state with targeted regression coverage plus a minimal shell key.
Consequences: later Scene Brain work should continue from this same queueable scene-launch seam when adding restore, recovery, or richer transition semantics. Selection UIs, scene diagnostics, and transition policies remain follow-up slices.
Status: accepted

---

Topic: early Scene Brain diagnostics should stay in the existing Jam and Log shell spine
Phase: Scene Brain
Question: once Riotbox has deterministic scene candidates and one committed `scene.launch` seam, what is the smallest honest next step that makes that state legible to the operator without opening a second scene page or a shell-only scene model?
Decision: surface active scene, next scene candidate, pending scene launch, and committed transport-scene context directly in the existing `Jam` overview and `Log` summary panels. Keep the slice presentation-only on top of the shipped app and runtime seam.
Why: the repo already keeps TR-909, MC-202, and W-30 seams visible inside the current shell spine. Scene Brain should become legible the same way before introducing richer scene launch, restore, or transition controls. A first diagnostic slice should deepen the current shell, not create a separate scene browser or debug page.
Evidence: `riotbox-app` now shows active scene plus next-candidate context in the Jam overview, folds scene state into the existing Log summary without adding a new page or panel family, and covers the new shell state with focused scene-diagnostic snapshot tests.
Consequences: later Scene Brain work should keep extending the same shell surfaces unless the roadmap explicitly calls for a fuller scene page. Replay-safe scene fixtures and richer launch or restore behavior remain follow-up slices.
Status: accepted

---

Topic: first Scene Brain recovery should reuse the committed restore pointer on the existing transport seam
Phase: Scene Brain
Question: once Riotbox already has deterministic scene candidates, one committed `scene.launch` seam, and visible scene diagnostics, what is the smallest honest next step that makes scene recovery real without opening a second transition model or scene browser?
Decision: add one bounded `scene.restore` action that targets the existing session `restore_scene` pointer, queues on `NextBar`, commits through the current action queue and transport boundary seam, and swaps the restore pointer back to the previously active scene when the restore lands. Keep richer transition shaping, scene recovery policy, and deeper restore diagnostics out of scope.
Why: the current contracts already name scene restore as part of the TUI and action lexicon, and `scene.launch` by itself still leaves Scene Brain without a real recovery path. The smallest honest move is to reuse the explicit restore pointer already present in session state instead of inventing a second scene stack, transition graph, or shell-only recovery model.
Evidence: `riotbox-app` now queues `scene.restore` on the same `NextBar` seam as `scene.launch`, blocks overlapping scene transitions, updates `active_scene`, transport `current_scene`, and `restore_scene` together on commit, exposes a minimal pending restore cue in the Jam shell, and covers both committed state and shell visibility with focused regressions.
Consequences: later Scene Brain work should continue from the same committed restore pointer when adding richer launch/restore cues, scene transition policy, or more musical recovery behavior. Replay-safe restore fixtures and more detailed shell diagnostics remain separate follow-up slices.
Status: accepted

---

Topic: first-run onramp should stay inside the existing Jam shell spine
Phase: Playable shell UX
Question: once Riotbox already has real lanes and actions, what is the smallest honest first-run improvement that helps a new user find one meaningful play moment without inventing a second onboarding shell or wizard?
Decision: add a bounded first-run onramp directly inside the existing Jam screen and help overlay. Show a reduced `Start Here` guidance block only while the session is still in an early state, and let it evolve from `start transport` to `first move queued` to `first change landed` instead of opening a separate first-run mode.
Why: the current user problem is not missing engine state, but missing orientation in the first 30 to 60 seconds. A small guidance layer on top of the shipped Jam shell preserves the current runtime and screen architecture, reduces equal-rank noise at first contact, and gives Riotbox one obvious first move without pretending the shell is already fully simplified.
Evidence: `riotbox-app` now swaps the dense source row for a dedicated `Start Here` block only during early first-run states, extends the help overlay with stage-aware onboarding hints, and covers both the untouched mature shell and the new first-run guidance with focused UI regressions.
Consequences: deeper perform-first Jam simplification and richer onboarding remain separate work. `RIOTBOX-94` can still reduce the long-term Jam surface, and later product work can still add a fuller first-run flow if the small inline onramp proves insufficient.
Status: accepted

---

Topic: Jam should become perform-first before Riotbox adds a separate inspect mode
Phase: Playable shell UX
Question: once Riotbox already has strong lane state, trust cues, and multiple support screens, what is the smallest honest next step that makes the Jam surface feel more like an instrument and less like an engine dashboard?
Decision: keep one Jam screen for now, but reduce it to a perform-first hierarchy: `Now`, `Next`, and `Trust` on the top row; three compact lane cards for `MC-202`, `W-30`, and `TR-909` in the middle; and a lower row containing only `Pending / landed`, `Suggested gestures`, and `Warnings / trust`. Move source detail, section lists, macro dumps, and deeper diagnostics off the main Jam surface and keep them on `Source`, `Capture`, `Log`, or the help overlay.
Why: the strongest UX feedback was not that Riotbox lacked state, but that too much equal-priority information was competing on the primary Jam surface. The next honest improvement is to reorder and reduce the surface, not to open a second inspect architecture before the simpler Jam hierarchy has been tested in use.
Evidence: `riotbox-app` now removes the old Source / Sections / Macros row from the main Jam surface, replaces it with three lane cards plus suggested-gesture and warning blocks, shortens the footer/help language toward primary versus secondary actions, and keeps fixture-backed Jam regressions green after the wording and layout shift. The review artifact at `docs/screenshots/jam_perform_first_baseline.txt` records the new hierarchy.
Consequences: a future inspect-mode split remains possible, but it is no longer the default next move. Follow-up UX slices should first test whether the reduced Jam surface plus better help text is enough. If further reduction still fails, a later ticket can add a deeper inspect layer without reopening the current screen contract.
Status: accepted

---

Topic: the first 30 seconds after ingest should bias toward one obvious success path
Phase: Playable shell UX
Question: after adding the bounded first-run onramp and reducing the Jam surface, what is the smallest next step that makes the first playable moment easier to discover without creating a second onboarding system?
Decision: keep the existing inline `Start Here` guidance, but sharpen it into one explicit first-success flow: `[Space]` to start transport, `[f]` to queue one first fill, and `[2]` to confirm the landed result in `Log`. Once the first action is armed or committed, keep the guidance focused on only the next decision: let it land, then either capture the keeper or undo it.
Why: the next user problem was no longer “what screen is this?” but “what do I do first, and how do I know it worked?” A bounded single-path flow is the smallest honest improvement. It preserves the current Jam/help architecture and gives new users one clearer first success before widening back out to the rest of the shell.
Evidence: `riotbox-app` now rewrites the first-run `Start Here` and help-overlay copy around a single first fill and confirmation loop, removes the earlier first-step ambiguity between multiple actions, and keeps the focused first-run UI regressions green.
Consequences: later first-run work can still introduce richer post-success guidance or broader onboarding, but the inline path should remain singular and easy to scan. If this still proves too open, the next follow-up should improve the moment-after-success guidance before inventing a separate onboarding surface.
Status: accepted

---

Topic: Jam should speak in gesture language on the perform-first surface
Phase: Playable shell UX
Question: once the Jam surface is reduced and the first-run path is tighter, what is the smallest next change that makes Riotbox feel less like an internal action model and more like an instrument?
Decision: keep the deep `Log`/diagnostic surfaces technically precise, but shift the perform-first Jam surface, footer, help overlay, and status line toward clearer gesture vocabulary. Use words such as `voice`, `jump`, `follow`, `hit`, and `push` where those improve immediacy, while leaving the internal action model and command ids unchanged.
Why: the remaining UX friction was not capability but wording. The Jam shell was still presenting several actions in engine terms (`role`, `scene select`, `trigger`, `reinforce`) even after the hierarchy was improved. Translating the outward-facing layer is the smallest honest next move because it changes how the shell reads without inventing a new behavior model.
Evidence: `riotbox-app` now updates status messages, footer/help guidance, Jam MC-202 card wording, and perform-facing pending/landed labels to use the curated gesture vocabulary while leaving the deeper `Log` diagnostics and action ids intact. The fixture-backed Jam shell regressions stay green after the wording change.
Consequences: future UX work should preserve the split between perform language and diagnostic language. If later tickets add an inspect surface, it can stay more technical; the default Jam surface should continue optimizing for musical intent first.

---

Topic: Jam inspect mode should deepen confidence without restoring the old dashboard
Phase: Playable shell UX
Question: after the perform-first surface and clearer gesture language both land, what is the smallest next step that adds confidence depth without re-bloating the default Jam screen?
Decision: keep the current perform-first Jam layout as the default, but add one explicit `perform / inspect` toggle inside `Jam`. The inspect view should preserve the top-level `Now / Next / Trust` frame while swapping the lower half for lane detail, source structure, material flow, and compact diagnostics. Do not create a second dashboard screen or hide this behind `Log`, `Source`, or `Capture`.
Why: the current UX gap is no longer missing information; it is missing confidence depth at the moment a user wants to look slightly further without leaving the Jam context entirely. A bounded inspect mode is the smallest honest follow-up because it preserves the reduced default surface and reuses existing app/runtime/source seams instead of reviving the older all-at-once Jam dump.
Evidence: `riotbox-app` now adds an explicit Jam inspect toggle, blocks that toggle during the first-run guided path, reuses the existing MC-202/W-30/TR-909 diagnostic lines plus source-graph and capture/runtime summaries, and keeps focused snapshot and key-handling tests green. The review artifact at `docs/screenshots/jam_inspect_mode_baseline.txt` records the bounded inspect hierarchy.
Consequences: later Jam UX work should keep the split clear: perform mode is the default instrument surface, inspect mode is a read-only confidence layer, and deeper technical truth should still live on `Log`, `Source`, and `Capture`. If later feedback asks for even more detail, the next move should be to refine inspect density, not to reopen a second hidden dashboard path.
Status: accepted

---

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

---

Topic: first workflow benchmarks should start as explicit interaction budgets, not a second measurement system
Phase: Playable shell UX
Question: after recent Jam UX work, what is the smallest honest way to start recording the roadmap's workflow benchmarks without inventing a new analytics or stopwatch architecture?
Decision: record the first workflow baseline as an explicit operator-path budget derived from the shipped example-source flow and the current quantization seam. For now, use the README/example-source path plus the first-run gesture path to document `time to first playable Jam state` and `time to first successful capture` in repo markdown under `docs/benchmarks/`.
Why: Riotbox needs benchmark visibility now, but the project does not yet need a second runtime measurement subsystem just to start tracking user-path budgets. The smallest honest move is to tie the benchmark to one shipped fixture, one shipped gesture path, and the current `NextBar` / `NextPhrase` commit model.
Evidence: the repo now has an explicit benchmark artifact at `docs/benchmarks/jam_workflow_baseline_2026-04-17.md`, grounded in the shipped `Beat08_128BPM(Full).wav` example path and the current first-run `Space -> f -> c` loop.
Consequences: later benchmark work should keep the same workflow names and fixture references, but can replace the derived timing budget with semi-automated or fully automated stopwatch data once that path exists. Until then, this benchmark family should stay small, readable, and tied to the shipped shell semantics.
Status: accepted

---

Topic: audio callback timing should be the live transport authority before deeper audio QA hardening
Phase: Pro Hardening
Question: once the app shell, queue, and audio render seams already exist, what is the smallest honest next move that makes live transport timing and quantized commit boundaries depend on the audio runtime instead of an app-side wall-clock pulse thread?
Decision: remove the app-local 20ms `RuntimePulseSource` thread and let `riotbox-audio` publish a typed timing snapshot derived from callback-owned beat progression. The app should consume that timing snapshot, reconstruct bar/phrase context from the current source graph, and commit queued actions from crossed boundaries observed in the audio-owned timing stream.
Why: the repo review finding was not about missing features; it was about musical honesty. As long as the app advanced transport and committed actions from its own wall-clock pulse, control-plane jitter could become the de facto timing spine while the callback merely rendered a lagging copy. Exposing callback-owned timing first is the smallest bounded fix because it moves authority toward the audio runtime without redesigning the whole scheduler in one slice.
Evidence: `riotbox-audio` now owns a small shared transport control/state seam and publishes `AudioRuntimeTimingSnapshot` values from callback progress; `riotbox-app` consumes those snapshots in the event loop, removes the old `runtime.rs` pulse thread module, and keeps focused transport/commit tests green under the new path.
Consequences: this does not yet finish the transport redesign. The app still reconstructs bar/phrase indices and still mirrors the current audio-owned beat position back into lane render state. Later hardening can push more of that contract into shared/core or audio-owned surfaces, but deeper audio QA and replay work now has one truthful live timing spine to build on.
Status: accepted

---

Topic: Capture target routing should use typed Jam view intent, not display strings
Phase: W-30 MVP
Question: once Capture guidance started using different next-step wording for W-30 pad targets and Scene targets, should the TUI branch on formatted labels such as `pad bank-a/pad-01`?
Decision: keep `CaptureSummaryView.last_capture_target` as the display label, but add a typed `last_capture_target_kind` projection for routing decisions. Capture `Do Next` and `hear ...` labels should branch on that kind and only use the display label for rendering.
Why: display wording is allowed to change as the TUI becomes more musical. If behavior-level guidance depends on string prefixes, a wording cleanup can silently change whether Riotbox offers W-30 audition/hit guidance or Scene confirmation guidance.
Evidence: `JamViewModel` now exposes `CaptureTargetKindView`; Capture `Do Next` and `capture_heard_path_label` branch on the typed kind while preserving the existing visible W-30 and Scene wording. Tests cover W-30 pad, Scene, and unassigned target projections.
Consequences: this is still a view projection over the existing `CaptureTarget` model, not a persistence change. Future Capture routing surfaces should consume typed view intent first and render display labels second.
Status: accepted

---

Topic: W-30 pending audition intent belongs in the Jam view model
Phase: W-30 MVP
Question: after Capture started explaining raw and promoted auditions in musical next-step language, where should raw-vs-promoted pending audition intent live?
Decision: project pending W-30 audition intent from the existing `ActionQueue` into `LaneSummaryView` as a typed view object containing kind, target, and quantization. The TUI may still render action ids in diagnostic surfaces, but Capture `Do Next` and compact lane cues should not reconstruct raw-vs-promoted audition state by scanning generic pending action command strings.
Why: raw and promoted auditions share the same `[o]` gesture but need different user-facing guidance. Keeping that distinction only as command strings in the generic pending list made the Capture surface fragile and duplicated classification logic in `riotbox-app`.
Evidence: `JamViewModel` now exposes `W30PendingAuditionView`; Capture `Do Next` renders queued raw/promoted audition guidance from that projection; focused tests cover raw and promoted pending audition kind, target, and quantization.
Consequences: this remains a presentation-model projection over the existing action system, not a second queue or W-30 action path. Future W-30 pending cue details should prefer typed Jam view projections when the perform-facing UI needs semantic intent beyond a generic command label.
Status: accepted

---

Topic: Capture handoff source readiness belongs in the Jam view model
Phase: W-30 MVP
Question: after Capture started showing compact `src` / `fallback` handoff cues, should the TUI derive that readiness by inspecting the latest session capture directly?
Decision: project Capture handoff readiness into `CaptureSummaryView` as typed view state. The TUI may still show detailed provenance in Capture inspect areas, but perform-facing handoff copy should consume the Jam view projection instead of scanning `session.captures`.
Why: `src` / `fallback` is not only raw provenance; it changes the user's confidence in `[w] hit` and `[p]->[w]` next steps. Keeping that decision in the view model keeps Capture guidance aligned with the typed projection pattern and prevents the TUI from accumulating more session-model branching.
Evidence: `JamViewModel` now exposes `CaptureHandoffReadinessView`; Capture `Do Next` and heard-path copy render the existing `src` / `fallback` wording from that projection; focused core and TUI tests cover fallback and source-backed readiness.
Consequences: this remains presentation-model state over the existing Capture model, not a persistence change or source-cache redesign. Future Capture confidence cues should prefer typed Jam view projections before adding more UI-side session inspection.
Status: accepted

---

Topic: MC-202 audio proof should start as an offline render seam before live mixer integration
Phase: MC-202 MVP / Audio QA
Question: after the routine audio audit found MC-202 state-proven but not musician-audible, should the next slice jump straight into live callback integration?
Decision: add a bounded offline MC-202 render seam and include one follower-vs-answer case in the lane recipe listening pack. Keep live TUI mixer integration out of this slice.
Why: the current MC-202 lane already has replay-safe role, follower, and answer state, but no audio contract. A small offline render seam gives the QA layer a real WAV and metric target without pretending the live instrument mix is finished or adding a hidden callback path too early.
Evidence: `riotbox-audio` now exposes `Mc202RenderState`, an offline render helper, unit coverage for distinct follower/answer output, and the lane recipe listening pack renders `mc202-follower-to-answer` with a minimum RMS delta.
Consequences: later MC-202 work should wire this typed render state into the app/audio runtime deliberately, with live mixer controls and TUI cues, rather than growing the offline proof into a shadow synth architecture.
Status: accepted

---

Topic: MC-202 live audio should consume the typed render seam instead of direct callback heuristics
Phase: MC-202 MVP / Audio QA
Question: once MC-202 has an offline render proof, what is the smallest honest step that lets a musician hear committed follower/answer state in the live Jam path?
Decision: thread `Mc202RenderState` through `riotbox-app` runtime projection, `AudioRuntimeShell` shared callback state, and the existing mixbuffer. Derive mode, routing, phrase shape, touch, transport, and music-bus level from committed session/runtime state, then render the bounded MC-202 bass voice beside TR-909 and W-30 without adding a second audio subsystem.
Why: the user-facing gap was that MC-202 gestures could be committed and logged while still not being part of the live sound. Reusing the typed render seam preserves queue/commit determinism and keeps the callback free of stringly role parsing or UI-only heuristics.
Evidence: `riotbox-app` now builds and exposes MC-202 render diagnostics from committed role/follower/answer state, `riotbox-audio` mirrors MC-202 render state through atomic shared runtime storage, and runtime tests prove the mixed buffer contains active MC-202 bass output.
Consequences: this is still a bounded first bass seam, not a finished MC-202 engine. Later MC-202 work should improve sound design, phrase continuity, live controls, and source-aware bass behavior on this same render path.
Status: accepted

---

Topic: MC-202 touch control should adjust the committed render seam directly
Phase: MC-202 MVP / Audio QA
Question: after MC-202 follower and answer state can be heard in the live mix, what is the smallest useful live control that does not create a second synth-control path?
Decision: expose `<` and `>` as bounded Jam controls for `runtime_state.macro_state.mc202_touch`. The controls refresh the existing typed `Mc202RenderState`, surface the current touch value in MC-202 diagnostics, and keep phrase generation plus role selection on the existing queue / commit seam.
Why: the musician needs one immediate performance parameter after a bass phrase lands, but Riotbox should not invent an ad hoc callback-only synth model. Touch is already persisted in session state and consumed by the renderer, so it is the safest first live control.
Evidence: `riotbox-app` now updates MC-202 touch through `JamAppState`, the shell maps `<` / `>` to that state refresh, app tests verify session/runtime-view projection, and `riotbox-audio` proves low-vs-high touch changes the same MC-202 phrase buffer metrics.
Consequences: future MC-202 live controls should follow this pattern: persisted macro or lane state first, typed render projection second, callback consumption third, and an output-path regression proving the audible seam changed.
Status: accepted

---

Topic: MC-202 phrase mutation should be quantized and render-state backed
Phase: MC-202 MVP / Audio QA
Question: after live touch control exists, how should Riotbox add the first MC-202 phrase mutation without opening a hidden sequencer or callback-only phrase path?
Decision: add `mc202.mutate_phrase` as a bounded `NextPhrase` action on the existing queue / commit seam. Commit writes an explicit MC-202 phrase variant into session lane state, projects that variant into the typed `Mc202RenderState`, and keeps direct live touch as the only immediate MC-202 macro control for now.
Why: Phase 4 requires quantized phrase mutation, but the safe first step is one replayable phrase variant, not a full phrase editor. Persisting the variant keeps replay/restore honest and lets audio QA prove the same lane state reaches the renderer.
Evidence: `riotbox-core` exposes pending MC-202 mutation and persists `phrase_variant`, `riotbox-app` queues and commits `mc202.mutate_phrase` from `G`, Jam/Log diagnostics show the variant, and `riotbox-audio` verifies `mutated_drive` differs from follower-drive output via delta-RMS and max-sample thresholds.
Consequences: future MC-202 phrase work should add richer variants or source-aware generation through the same committed lane-state seam before adding any editor or MIDI-style sequencer surface.
Status: accepted

---

Topic: MC-202 recipe proof should use signal-delta listening cases, not only loudness deltas
Phase: MC-202 MVP / Audio QA
Question: now that MC-202 has live touch and phrase mutation, how should the recipe-level QA pack prove those gestures are audible and not just visible in state/logs?
Decision: extend the lane recipe listening pack with explicit `mc202-touch-low-to-high` and `mc202-follower-to-mutated-drive` cases, and require sample-by-sample signal delta RMS alongside normal RMS delta.
Why: touch changes are partly loudness/energy changes, but phrase mutations can remain similarly loud while still being musically different. A plain RMS comparison can miss identical-output or fallback-collapse bugs when two phrases have similar energy. Signal-delta RMS catches actual waveform difference and writes the evidence beside the WAVs.
Evidence: `lane_recipe_pack` renders seven cases, writes signal delta metrics into comparisons and pack summaries, and the local packs passed with MC-202 touch signal delta RMS `0.006608`, pressure signal delta RMS `0.009436`, and mutated-drive signal delta RMS `0.010100`.
Consequences: future listening-pack cases should prefer paired signal-delta checks when the musical claim is "different phrase or gesture", and use plain RMS only as an additional energy sanity check.
Status: accepted

---

Topic: MC-202 pressure should be a quantized role phrase, not a free-running bass layer
Phase: MC-202 MVP / Audio QA
Question: how should Riotbox add the first explicit pressure behavior without turning the MC-202 lane into an unbounded sequencer?
Decision: add `mc202.generate_pressure` on the existing `NextPhrase` queue / commit seam. The commit stores role `pressure`, clears phrase variants, raises MC-202 touch to a bounded pressure value, and projects to a typed `Pressure` render mode with a sparse `pressure_cell` phrase shape.
Why: Phase 4 asks for pressure and identity without overplaying. A quantized pressure role gives the performer one clear gesture for offbeat pressure while preserving replayability, undo/log visibility, and the same render-state path as follower, answer, touch, and phrase mutation.
Evidence: `riotbox-core` exposes the new action and pending cue, `riotbox-app` queues/commits `P` pressure through the existing MC-202 phrase-control path, and `riotbox-audio` plus the lane recipe listening pack prove `pressure_cell` differs from follower drive with signal delta RMS `0.009436`.
Consequences: future pressure work should add note-budget/source-aware policy on top of this committed role seam rather than adding callback-only heuristics or a separate MC-202 phrase editor.
Status: accepted

---

Topic: MC-202 instigator should be a quantized role phrase on the existing render seam
Phase: MC-202 MVP / Audio QA
Question: how should Riotbox add the missing `instigate` behavior from the feral addendum without opening a second MC-202 engine or free-running sequencer?
Decision: add `mc202.generate_instigator` on the existing `NextPhrase` queue / commit seam. The commit stores role `instigator`, clears phrase variants, raises MC-202 touch to a bounded instigator value, and projects to a typed `Instigator` render mode with an `instigator_spike` phrase shape.
Why: the MC-202 lane needs one sharper push gesture in addition to follower, answer, and pressure. Keeping it as a committed phrase role preserves replayability, pending/commit visibility, undo/log behavior, and the existing app-to-audio typed render path.
Evidence: `riotbox-core` exposes the new action and pending cue, `riotbox-app` queues/commits `I` instigator through the existing MC-202 phrase-control path, app/UI fixtures cover committed state and shell output, and `riotbox-audio` plus the lane recipe listening pack prove `instigator_spike` differs from follower drive with signal-delta thresholds.
Consequences: future instigator work should add note-budget and source-aware contour policy on top of this committed role seam, not as callback-only heuristics or a separate MC-202 phrase editor.
Status: accepted

---

Topic: MC-202 anti-overplay should be a typed note budget on the render seam
Phase: MC-202 MVP / Audio QA
Question: what is the smallest note-budget policy that reduces MC-202 clutter without inventing a phrase editor, source-aware scorer, or callback-only heuristic?
Decision: add `Mc202NoteBudget` to the typed MC-202 render state and derive it from the committed phrase shape. `pressure_cell` uses a sparse budget, `instigator_spike` uses a push budget, `mutated_drive` keeps a wider budget, and follower/answer/root phrases use a balanced budget.
Why: the lane needs anti-overplay behavior, but the first policy should stay deterministic and replay-aligned. Putting the budget in render state makes it visible to app projection, shared audio runtime state, tests, and listening-pack metrics without creating a second phrase system.
Evidence: `riotbox-audio` now caps active steps per 16-step cycle from `Mc202NoteBudget`, tests prove the balanced budget reduces density without silencing follower drive, `riotbox-app` projects the budget from committed phrase shape, and the lane recipe pack passes after recalibrating touch/instigator thresholds for the less-dense output.
Consequences: future source-aware contour and phrase scoring work should choose or adjust this typed budget from analysis/scene context rather than bypassing it with ad hoc callback gating.
Status: accepted

---

Topic: MC-202 contour hints should be section-derived render state, not extracted melody
Phase: MC-202 MVP / Audio QA
Question: what is the smallest source-aware contour step that moves MC-202 beyond static phrase shapes without inventing pitch tracking or a phrase editor?
Decision: add `Mc202ContourHint` to the typed render state and derive it from the current projected source/scene section. Build sections lift, high-energy drop/chorus sections drop, break/intro/outro or low-energy sections hold, and unknown sections stay neutral.
Why: Phase 4 asks for contour following with feral simplification. A coarse section-derived hint gives the lane source awareness while preserving replayable committed roles, deterministic app projection, and callback-safe rendering.
Evidence: `riotbox-app` projects the hint from source sections into runtime diagnostics, `riotbox-audio` mirrors it through shared runtime state and changes phrase intervals in the renderer, and the lane recipe pack proves `mc202-neutral-to-lift-contour` differs from neutral follower drive with signal delta RMS `0.007847`.
Consequences: later hook-response and source-aware phrase scoring should refine this typed hint instead of bypassing it with callback-only heuristics or full melody extraction.
Status: accepted

---

Topic: MC-202 hook response should be explicit answer-space restraint
Phase: MC-202 MVP / Audio QA
Question: how should MC-202 avoid doubling hook-like sections without adding a phrase editor or Ghost-driven composition?
Decision: add `Mc202HookResponse` to the typed render state and derive `answer_space` for follower/leader roles when the current source/scene section is hook-like, currently chorus labels or hook/chorus tags. `answer_space` uses a sparse note budget and offbeat response gating in the renderer.
Why: the feral addendum asks for hook-response rules instead of hook doubling. Making the rule explicit in render state keeps it visible to TUI diagnostics, shared audio state, tests, and listening-pack metrics while avoiding callback-only heuristics.
Evidence: `riotbox-app` projects chorus/hook context to `Mc202HookResponse::AnswerSpace`, `riotbox-audio` renders answer-space with lower density and offbeat offsets, and the lane recipe pack proves `mc202-direct-to-hook-response` differs from direct follower drive with signal delta RMS `0.008681` and RMS delta `0.004777`.
Consequences: future source-aware phrase scoring can refine which sections are hook-like, but it should keep using the typed hook-response seam rather than hiding hook restraint in ad hoc phrase substitution.
Status: accepted

---

Topic: MC-202 recipe QA should replay the musician flow, not only isolated pairs
Phase: MC-202 MVP / Audio QA
Question: after individual MC-202 gestures have output proofs, what is the smallest test that proves the documented recipe path works as a sequence?
Decision: add an app-level recipe replay regression that queues and commits follower, answer, pressure, instigator, phrase mutation, and touch adjustment, then renders each landed `Mc202RenderState` through the audio seam and compares successive buffers.
Why: isolated pair tests can pass while the musician-facing flow still breaks through ordering, queue/commit state, or render projection drift. A bounded replay test catches sequence-level regressions without requiring full TUI automation or realtime device capture.
Evidence: `mc202_recipe_replay_proves_control_and_audio_path` verifies queue/commit state, landed render modes/shapes, non-silent MC-202 buffers, and signal-delta thresholds across the current Recipe 2 gesture chain.
Consequences: broader recipe replay and observer correlation should build from this pattern: drive the same actions a musician performs, assert the control path, then prove the nearest audio seam changed.
Status: accepted

---

Topic: MC-202 MVP exit requires state and audio undo, not log-only undo
Phase: MC-202 MVP / Replay QA
Question: after follower, answer, pressure, instigator, touch, mutation, contour, note budget, hook response, and recipe replay proof exist, what still blocks calling the first MC-202 MVP exit-clean?
Decision: require a bounded MC-202 undo rollback slice before closing the MVP phase. Undo must restore the previous MC-202 lane state, refresh the typed render state, and prove the rendered output returns to the previous audible seam; marking the action log entry as `undone` is not enough.
Why: the phase definition explicitly requires replay and undo to remain intact. Riotbox is an instrument for trying moves and backing out; if a musician hears an MC-202 move and presses undo, the sounded lane state must roll back, not only the diagnostic history.
Evidence: the current `undo_last_action` path marks the latest undoable action as undone and appends an undo marker, while MC-202 side effects are already committed into session lane state, macro touch, and typed render projection.
Consequences: complete the rollback on the existing action/log/session/render seam. Do not introduce a second MC-202 history stack or callback-only undo path.
Status: accepted

---

Topic: MC-202 undo should restore session-backed lane snapshots and audio output
Phase: MC-202 MVP / Replay QA
Question: what is the smallest undo implementation that makes MC-202 experimentation musically reversible without introducing a second phrase system?
Decision: store bounded MC-202 undo snapshots in `runtime_state.undo_state` at commit time, keyed by action id. When undo targets an MC-202 phrase action, restore the previous role, phrase reference, phrase variant, and touch from that snapshot, then refresh the typed render projection.
Why: undo has to change the sounded lane state, not just mark a history row as undone. Keeping the snapshot in session runtime state makes the rollback explicit and serializable while avoiding callback-local memory or a separate MC-202 history stack.
Evidence: `undo_mc202_phrase_move_restores_lane_state_and_audio_path` commits follower then answer, undoes answer, verifies the action log and Jam lane return to follower state, and proves the post-undo render buffer matches the previous follower buffer while differing from the undone answer buffer. `session_file_roundtrips_via_json` also covers persisted MC-202 undo snapshots.
Consequences: future lane-specific undo should use the same pattern: capture bounded pre-state before applying committed side effects, restore it through session state, and prove the nearest audible seam changed back.
Status: accepted
