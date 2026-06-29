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

### 7.1 Source monitor playback

Source monitor playback is a transport feature, not an external player.

Rules:

- decode and normalize the source outside the realtime callback
- hand the callback only prepared PCM buffers and bounded realtime-safe state
- map transport position to source frame position through the selected timing /
  source-time contract outside expensive callback work
- source seek updates callback-consumable cursor state without file I/O
- monitor presets are `source`, `blend`, and `riotbox`
- source end behavior clamps / stops by default; looping or wrapping requires an
  explicit future mode
- the source monitor must not mask weak generated-lane QA; source-only and
  source-layer states must be explicit

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
- MC-202 renders only trusted `source_phrase_plan` material on the music bus.
  Primitive phrase-shape labels may remain compatibility / diagnostic state,
  but they must not produce hardcoded musical fallback output when no
  source-derived plan exists.

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

### 11.3 Callback hot-path contract

The normal realtime callback path must not perform:

- file I/O
- JSON parsing or serialization
- string formatting for product logs
- sidecar or analysis calls
- UI work
- blocking locks
- unbounded allocation or repeated buffer growth

Buffer allocation and expensive state preparation belong on the control side.
If the audio backend delivers variable buffer sizes, Riotbox should use a
documented scratch strategy or perform controlled resizing outside the normal
hot path.

The callback hot path should be documented before adding more device-facing
features, so future reviews can distinguish safe realtime work from control
plane preparation.

### 11.4 Coherent render-state snapshots

Realtime render state must be read as a coherent snapshot, not as an accidental
mix of old and new independent atomics.

Acceptable strategies include:

- a revisioned double buffer where the control thread writes a complete inactive
  state and then swaps the active revision
- a seqlock-style snapshot where the audio thread detects partial updates
- another documented lock-free or bounded handoff that preserves coherence

Current implementation:

- shared realtime render-state groups use a bounded seqlock-style revision
  marker
- control-plane updates mark the group revision odd before field writes and even
  after the complete group is written
- the realtime callback attempts a small fixed number of stable reads
- if a stable read is unavailable because a control update is active, the
  callback reuses the last complete snapshot for that group instead of rendering
  a mixed old/new state
- the callback must not spin without bound, block on locks, allocate, or call
  the control plane while waiting for coherence

Tests should cover partial-update and revision-mismatch cases before this
becomes a broad lane-control surface.

### 11.5 Gain staging and offline/realtime parity

Riotbox mixes source monitor, TR-909, MC-202, W-30, resample, and future FX
material. The mix contract must keep this loud and physical without hiding
failure modes.

Rules:

- use consistent gain conversion for lane and bus levels
- add master-bus clipping protection or soft limiting where needed
- measure peak, RMS, DC offset, and clip count in relevant offline reports
- do not let master processing mask weak source-character or fallback-collapse
  evidence
- converge offline and realtime rendering around shared render functions where
  practical

Current metric contract:

- `OfflineAudioMetrics` reports `peak_abs`, `rms`, `dc_offset`,
  `clip_count`, `near_clip_count`, and `headroom_to_full_scale`
- `clip_count` counts samples at or beyond full scale
- `near_clip_count` counts samples at or beyond the `0.98` near-clip threshold
- `headroom_to_full_scale` may be negative when a render exceeds full scale;
  reports must not hide that with post-hoc WAV clamping

Current limiter policy:

- no global master limiter is applied in the product render path yet
- report pre-clamp metrics before WAV writers clamp to PCM range
- treat `clip_count > 0` as a reportable controlled-clipping or failure signal,
  not as something to hide with output-file clipping
- add soft limiting only through a future shared master-bus seam that proves it
  preserves weak-output, source-character, and fallback-collapse gates

Offline and realtime-simulation renders should become comparable under the same
state, with explicit tolerances where backend buffer boundaries or floating
point differences make bit identity unrealistic.

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
