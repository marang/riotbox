# Riotbox Audio Core Spec

Version: 0.1  
Status: Draft  
Audience: realtime, DSP, QA, product

---

## 1. Purpose

This document defines the core realtime audio contract for Riotbox.

It exists so that:

- audio behavior stays stable under live use
- analysis, Ghost, and UI cannot destabilize playback
- device lanes operate inside one measurable timing model
- tests and benchmarks have a concrete target

---

## 2. Core Rule

The audio callback is sacred.

No analysis work, Ghost reasoning, file I/O, or UI rendering may block or stall the realtime audio path.

If a subsystem cannot meet that rule, it must move out of the realtime path.

---

## 3. Responsibilities

The audio core owns:

- device initialization and teardown
- audio callback scheduling
- transport timing
- bus graph execution
- voice and lane rendering
- quantized action application at commit boundaries
- meter and health telemetry

It does **not** own:

- deep analysis
- Ghost planning
- session authoring semantics
- TUI page behavior

---

## 4. Design Goals

- stable callback timing
- predictable latency behavior
- bounded CPU behavior
- clear separation between realtime and non-realtime work
- measurable health signals from day one

---

## 5. Core Runtime Model

Riotbox audio should be organized around:

1. transport clock
2. quantization scheduler
3. lane renderers
4. mixer bus graph
5. capture taps
6. telemetry

This model must stay readable. Cleverness belongs in musical logic, not in opaque callback structure.

---

## 6. Threading Boundaries

Minimum thread model:

- realtime audio thread
- control thread or runtime coordinator
- analysis / sidecar thread or process boundary
- UI thread

Rules:

- the realtime thread may consume lock-free or bounded handoff structures
- the realtime thread must not wait on sidecar or UI
- action commits should be prepared outside the callback and applied at safe boundaries

---

## 7. Transport and Timing

The audio core must expose:

- playback state
- sample time
- musical position
- beat / bar boundary awareness
- quantization commit points

Transport is the timing authority for playback.

The Source Graph informs musical alignment, but callback timing belongs to the audio core.

---

## 8. Quantized Action Commit

The audio core must support commit points for actions defined in the Action Lexicon.

Minimum commit targets:

- immediate
- next beat
- next half bar
- next bar
- next phrase

Rules:

- action resolution happens before the boundary when possible
- commit application at the boundary must be bounded and lightweight
- rejected or delayed actions must surface clearly to the control layer

---

## 9. Lane Model

The MVP audio core must support at least three musical lane types:

- `lane_tr909`
- `lane_mc202`
- `lane_w30`

And supporting buses:

- source bus
- drum bus
- music bus
- FX sends / returns
- capture taps
- master bus

Rules:

- lane ownership of sound generation is clear
- cross-lane influence happens through explicit control or bus routing, not hidden coupling

---

## 10. Bus Graph

The bus graph should support:

- level control
- mute / solo where needed
- send / return routing
- light insert processing
- capture points

MVP expectation:

- enough routing to support instrument identity
- not a full DAW mixer abstraction

---

## 11. Capture Path

Capture is core product behavior, not an afterthought.

The audio core must support:

- quantized capture start
- capture end rules
- internal resample taps
- provenance-friendly capture IDs

Rules:

- capture must not break playback
- capture timing must be aligned with transport
- capture events must be visible to session and action systems

### 11.1 Source audio cache seam

Raw capture playback must not read or decode files from the realtime callback.

The bounded early seam is a non-realtime source-audio cache:

- decode source WAV fixtures before callback use
- store normalized interleaved `f32` samples with explicit sample rate and channel count
- expose bounded sample-window access for source-backed W-30 preview paths
- project a small fixed-size preview window from `CaptureRef.source_window` into callback-safe W-30 preview state
- write committed source-window captures as PCM16 WAV artifacts outside the realtime callback when the app has a session path and decoded source cache
- prefer loaded committed capture artifacts for focused W-30 pad preview / trigger state, falling back to source-window projection when no artifact is available
- current W-30 preview window size is `2048` mono samples, deliberately bounded so it can sound more like captured material without becoming full callback-side sample streaming
- keep cache loading and source-window projection outside the realtime callback

Current limitation:

- the initial cache supports PCM 16-bit and PCM 24-bit WAV fixture input
- committed source-backed capture artifacts are PCM16 WAV files for the first app path
- the current raw, promoted, and recall preview paths use a bounded preview excerpt, not a full pad-bank sampler engine
- focused pad playback is still one bounded preview seam, but it now consumes capture artifacts when available instead of treating `storage_path` as decorative metadata
- broader codec support and true W-30 sample playback remain separate implementation slices

### 11.2 Bounded W-30 internal bus print seam

The first real W-30 internal resample print should be an offline app/control-plane operation, not a realtime recorder.

Smallest acceptable seam:

- input: one committed W-30 capture selected by the current lane focus
- source audio: prefer the committed capture artifact, then fall back to source-window projection only when no artifact exists
- render policy: use the existing W-30 preview / resample-tap state as the audible processing policy, including music-bus level, grit, source profile, and generation depth
- duration: bounded to the input capture duration or a documented maximum window for MVP safety
- output: a new PCM16 WAV artifact at the derived resample capture `storage_path`
- session result: a new `CaptureRef` with `CaptureType::Resample`, explicit `lineage_capture_refs`, incremented `resample_generation_depth`, and no direct `source_window` unless the printed artifact is still a literal source-window copy
- Feral policy cue: when the loaded source graph has Feral break-support evidence and the resample keeps explicit lineage, the committed action result and capture note should expose this as a lineage-safe W-30 Feral rebake / reuse decision; if the same graph carries high quote-risk evidence, the cue must be held rather than approved
- realtime rule: the audio callback must never write files or perform offline bus prints

Minimum QA gate:

- control-path test for queue -> commit -> new resample capture -> lane focus
- artifact test proving the printed WAV exists, reloads, has expected sample rate / channel count / bounded duration, and is not silent
- comparison against raw source/capture artifact and synthetic fallback so the bus print cannot silently collapse to either control
- docs or PR notes must state that full multitrack recording and export remain out of scope

---

## 12. Health Telemetry

The audio core must publish measurable health data.

Minimum telemetry:

- callback duration
- worst callback spike
- xrun count
- underrun count if available
- CPU estimate or processing load proxy
- action queue lag

These metrics must be visible to benchmarks and, where useful, the TUI.

---

## 13. Failure Handling

Failures must degrade safely.

Examples:

- analysis unavailable: continue playback
- Ghost unavailable: continue playback
- provider timeout: keep current state
- failed capture: report failure without destabilizing transport

The product may become less capable. It must not become musically dishonest or unstable.

---

## 14. MVP Requirements

Audio Core v1 must support:

- one reliable output path
- playback of source-derived and generated material
- quantized action commits
- basic per-lane mixing
- capture taps
- health telemetry

It does not yet need:

- advanced offline render graph
- plugin-host abstraction
- large-scale modulation matrix
- ambitious spectral or granular engines

---

## 15. Validation Requirements

Required validation:

- playback start / stop tests
- commit-boundary tests
- capture timing tests
- stable callback metrics in baseline environment
- soak tests for longer jam sessions

Benchmark tie-ins:

- callback timing
- xrun count
- action queue lag
- time to first playable audio state

---

## 16. Open Follow-Ups

This draft should be followed by:

1. backend selection decision
2. transport implementation decision
3. exact bus graph layout
4. capture buffer and file-write policy
