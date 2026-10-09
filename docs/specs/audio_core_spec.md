# Riotbox Audio Core Spec

Version: 0.1  
Status: Draft  
Audience: realtime, DSP, QA, product

---

## 1. Purpose

This document defines the core realtime audio contract for Riotbox.

For the meaning and change rules of numeric measurements, limiter boundaries,
DSP coefficients, and recipe parameters, see
[`audio_numeric_values.md`](../engineering/audio_numeric_values.md).

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

### 7.1 Transport position and musical-index convention

- `AudioRuntimeTimingSnapshot.position_beats` and source-monitor anchor positions
  are zero-based continuous cursors: `0.0` is the start of the first beat and
  `4.0` is the start of bar 2 in 4/4.
- Session V1 `TransportClockState.beat_index` and
  `CommitBoundaryState.beat_index` remain zero-based integral transport
  cursors. Their `bar_index` and `phrase_index` are one-based musical
  identities. Current code must not reinterpret old boundary values.
- Source Graph `BeatPoint.beat_index` is a separate one-based grid identity.
  A transport cursor must add one only when looking up a Source Graph beat;
  BPM fallback math continues to use the zero-based cursor directly.
- The canonical 4/4 conversion therefore maps cursor `4.0` to beat index 4 /
  bar 2, cursor `8.0` to beat index 8 / bar 3, and cursor `16.0` to beat index
  16 / bar 5 / phrase 2 (with the established four-bar phrase fallback).
- Code crossing between these representations must use the shared transport
  conversion helpers. It must not reuse a one-based Source Graph beat identity
  as a zero-based source-monitor or callback cursor.

### 7.2 Source monitor playback

Source monitor playback is a transport feature, not an external player.

Rules:

- decode and normalize the source outside the realtime callback
- hand the callback only prepared PCM buffers and bounded realtime-safe state
- publish source PCM ownership, monitor mode/gains, and source anchors as one
  immutable atomic snapshot; control-only updates must explicitly retain the
  current source, while only a named source-replacement operation may install
  new PCM or make source availability absent
- source-snapshot reads in the callback must remain lock-free and allocation-free;
  RBX-427 uses a preallocated single-producer/single-consumer triple buffer with
  a unique callback reader borrowing an immutable payload. Control writers
  retain the authoritative latest snapshot separately from recycled slots.
  Replaced slot payloads and their PCM ownership are reclaimed on the control
  side, never by callback-invocation Arc cloning/dropping. Retained slots may
  delay reclamation until later publication or shutdown; the slot count does
  not bound total source/cache memory
- map transport position to source frame position through the selected timing /
  source-time contract outside expensive callback work
- source seek updates callback-consumable cursor state without file I/O
- a prepared source may differ from the output-device sample rate; the callback
  performs only bounded, allocation-free interpolation from the prepared PCM
  buffer and must not mark a valid 44.1 kHz source unavailable on a 48 kHz
  device (or vice versa)
- monitor presets are `source`, `blend`, and `riotbox`
- when source material is genuinely unavailable, `source` is explicitly silent
  and degraded while `blend` preserves its real Riotbox-lane component and is
  still surfaced as degraded; it must never claim that source audio is present
- source end behavior clamps / stops by default; looping or wrapping requires an
  explicit future mode
- monitor mode changes, transport start/stop, and source-anchor jumps retain
  callback-local transition state and use a short allocation-free gain ramp or
  source-cursor crossfade; source EOF fades over at most 5 ms (and no more than
  one sixteenth of a very short source) instead of producing a hard edge
- an explicit play, pause, or stop request remains authoritative until the audio
  runtime reports the requested running state; an older callback timing snapshot
  must not reverse the musician's command
- a running-to-stopped transition silences every active lane through a bounded
  callback-local fade of at most 5 ms and remains silent until transport resumes
  or the musician deliberately starts a stopped manual preview
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
- MC-202 source-phrase renderer v2 maps each callback transport position to
  the nearest absolute audio frame and derives in-buffer musical time from that
  integer frame. Equivalent timelines must therefore be sample-exact across
  callback partitions; callback size must not alter oscillator phase, phrase
  selection, or envelopes.
  Primitive phrase-shape labels may remain compatibility / diagnostic state,
  but they must not produce hardcoded musical fallback output when no
  source-derived plan exists.

### 9.1 Source-backed live performance projection

For the trusted live path, the control plane may derive one typed
`LivePerformancePolicy` from the confirmed source timing, the current Source
Graph, and the committed MC-202 `source_phrase_plan`. This is a shared render
projection consumed by W-30, TR-909, and MC-202; it is not a second arrangement,
action, or persistence system.

Rules:

- the requested MC-202 role first gates eligible source-derived families:
  `pressure` -> `sub_pressure_shove`, `instigator` ->
  `fill_pickup_instigator`, `answer` -> sparse-answer / callback /
  hook-restraint, and `leader` / `follower` -> callback; if none survives, the
  lane projects explicit degraded silence instead of another audible intent
- within that compatible set, the selected MC-202 candidate family determines
  the audible intent (`bass_pressure`, `punctuate`, `instigate`, or `stay_out`)
- bass ownership is explicit: only a trusted `sub_pressure_shove` candidate may
  assign it to MC-202; all other candidates report `unassigned` rather than
  implying that a weak answer or pickup is the bass lane
- the policy may set bounded W-30, TR-909, and MC-202 level/touch/slam floors so
  the selected all-lane hierarchy survives the live mixer
- measured phrase-audio evidence may select one typed held-state character:
  `dense_break`, `tonal_hook`, or `sparse_pressure`. Selection uses a named
  normalized contrast margin across spectral brightness, low/mid body,
  offbeat onset density, and hook-restraint evidence; source paths or filenames
  must never control the branch. Missing/untrusted evidence preserves the
  human-passed `dense_break` default
- `tonal_hook` promotes the source-backed W-30 capture and assigns both TR-909
  and a generic fill-pickup MC-202 decision to explicit `stay_out` for the held
  state. An explicit committed Fill or Scene movement may still override that
  default. `sparse_pressure` assigns drum/transient impact to
  TR-909, keeps the W-30 source rhythm audible, and converts that generic pickup
  to bounded punctuation. A trusted explicit bass/answer/stay-out candidate
  retains its own role; character policy must not steal or invent bass ownership
- held-state character defaults may select TR-909 pattern adoption and phrase
  variation only for SourceSupport/BreakReinforce. Explicit committed Fill and
  Scene-movement vocabulary keeps precedence
- character-aware destructive intent must remain source-backed and
  grid-coherent. Dense/tonal may use a bounded pitch drag. Sparse destructive
  playback must remain at `1.0x` and may gate each source-derived chop inside
  its trigger step; it must not accelerate an already percussive source so its
  kicks drift between the fixed-grid TR-909 hits
- those shared floors must preserve headroom for explicit performer gestures;
  a fill, slam, trigger, launch, or restore must project a distinct bounded
  articulation instead of collapsing into the policy baseline
- the live TR-909 Fill may derive one callback-local `FillFocus` articulation
  from the existing typed render projection; it is active only for a running,
  audible `Fill` on `DrumBusSupport` and is not new Session, replay, action, or
  app state
- when the trusted source timing supplies a non-zero bar phase, the shared live
  policy projects that confirmed zero-based anchor into the TR-909 render state.
  Both the Fill recipe step grid and `FillFocus` subtract the same anchor before
  evaluating their bar-local positions. Missing or non-finite derived anchors
  retain legacy zero-phase behavior; the audio crate must not infer a separate
  downbeat
- for the supported `MainlineDrive + PhraseDrive` Fill, `FillFocus` uses one
  sample-position-derived, click-safe envelope to lower the non-TR-909 music
  bed for the final half bar. Its bounded pre-ramp begins before beat three so
  the bed is absent when the drum-owned call starts, stays absent through the
  beat-four rush and choke, and releases after the late DiveStomp at beat
  `3.75`; it is not a second mute or arrangement system.
  `blend` applies the envelope to its source layer, `riotbox` applies it only
  to W-30 / MC-202 / resample material, and `source` remains sample-identical
  and unaffected
- `FillFocus` leaves the TR-909 signal unchanged and returns fully at the next
  bar. A versioned Fill recipe may own bounded output trim for its own fixed
  voice sum so the exact clean path does not depend on the master limiter; that
  trim must not alter historical recipe IDs. Focus and recipe gain remain
  deterministic across realtime callback partitions and the exact offline
  RuntimeMix seam; a silent or wrongly routed drum lane must never duck the
  arrangement
- the supported dense-break `MainlineDrive + PhraseDriveHardCut` Fill uses fixed
  callback-local, independently decaying kick, snare-body/noise, and
  metallic-hat voices. The current
  `PhraseDriveBreakCutStompV2` four-beat arc has `1 / 2 / 6 / 1` sounding
  events. V2 transfers the non-TR-909 bed to silence from the start of the Fill
  bar through a short click-safe fade, so the fixed drum call reads as one
  deliberate role takeover rather than being pasted onto a differently phased
  promoted source hook. Beat three becomes a syncopated kick/snare/hat call;
  beat four chokes on its downbeat,
  remains empty for six 32nd-note slots, then answers with a late pitch-dive
  kick+snare flam. The choke removes existing voice tails through a bounded
  click-safe callback-local release but is not a sounding trigger. At 130 BPM
  the resulting negative-space window is about 346 ms and must read as an
  arrangement event rather than a micro-rest.
  Historical `PhraseDriveChokeDiveStompV1` and
  `PhraseDriveLongChokeDiveStompV2`, and `PhraseDriveBreakCutStompV1` remain
  registered listening controls but are no longer selected by the current
  Mainline Golden Path. The historical `MainlineDrive + PhraseDrive` selection
  remains on `PhraseDriveBreakCutStompV1`, including its original half-bar
  focus and unity recipe gain; the V2 whole-bar focus and local output trim are
  not a global rewrite of that render-policy pair
- the fixed Fill arc, its paired `FillFocus`, and its voice triggers come from
  one typed, versioned callback-safe recipe authority. This recipe is a
  versioned `primitive_renderer` Golden-Path vocabulary: the explicit committed
  performer Fill gesture may use it on the product path, while confirmed source
  evidence makes the path available and supplies timing but does not yet select
  or compose its rhythm or articulation. It therefore proves live instrument
  reachability, not source-derived musical intelligence; the manifest must name
  the selected recipe and inputs, and that promotion requires a later
  source-evidence-owned recipe-selection slice
- the signature DiveStomp must change articulation rather than only trigger
  gain: its kick falls from a high drum onset toward low kick body, its snare
  schedules one deterministic delayed crack, and the preceding choke plus
  deeper `FillFocus` pocket must remain audible in the exact RuntimeMix Blend.
  These fixed-size counters and envelopes are private realtime state derived
  from the typed existing policy, never Session, replay, or action truth
- this is a Golden Path contract, not evidence that `PhraseDrive` already
  expresses distinct musical ownership for every pattern-adoption profile;
  TakeoverGrid-specific Fill orchestration remains a later audible slice
- the Fill voice sum releases to zero at the bar edge and clears hidden voice
  tails before the next downbeat. Fill-to-Fill, Fill-to-non-Fill, exact
  one-subdivision seeks, and 127/128-frame callback partitions must neither
  restore stale tails nor leak the Fill subdivision into the legacy renderer.
  Non-Fill TR-909 modes stay sample-identical to their established composite
  path
- the policy remains unavailable without matching confirmed timing, a trusted
  dense-break window, and a matching committed MC-202 source-phrase decision;
  a committed degraded / fallback decision projects `stay_out` rather than
  reviving the originally requested role
- unavailable or unassigned ownership must not synthesize replacement bass or
  other fallback music
- when a committed MC-202 plan has typed `source_section_id` ownership and a
  landed Scene targets another section, MC-202 must stay silent until a trusted
  plan for that section exists; the live performance policy must report bass
  ownership as unassigned and must not reuse the previous section's expression
  floors for TR-909 or W-30. Provenance labels must not control this branch
- Scene launch may install a source-backed lane projection, while Scene restore
  must recover the projection paired with the restored scene. A documented
  changed return may commit `scene.restore` and performer-owned W-30 damage at
  the same bar boundary, but QA must separately prove the restore-only scene
  projection, prove that non-W-30 lanes stay identical, and attribute the
  additional audible delta to the committed W-30 action. The latest restore
  transition remains anchor/observer truth but is not itself a new persistent
  lane profile.
- Scene Source Monitor repositioning uses the target section's matching primary
  bar-grid downbeat. Raw section timestamps are descriptive evidence, not a
  playable anchor; a missing matching primary-grid bar leaves repositioning
  unavailable rather than introducing an off-grid source jump.

The persisted Source Graph, timing confirmation, and MC-202 source phrase plan
remain replay truth. `LivePerformancePolicy` is rederived after load/replay from
that truth so the callback receives a coherent render snapshot without storing
another mutable product truth.

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
- the audible internal resample tap activates only for a focused
  `CaptureType::Resample` with non-empty capture lineage and positive resample
  generation depth; ordinary loop/pad capture, audition, promotion, recall, and
  trigger paths keep the tap idle and silent
- `promote.resample` owns the first audible tap activation. The presence of any
  capture or W-30 lane focus is not sufficient activation evidence for the
  source-backed tap. A lineage-ready resample capture without a hydrated audio
  artifact reports `source_audio_unavailable`, routes silent, and must not
  synthesize replacement audio.

### 11.1 Source audio cache seam

Raw capture playback must not read or decode files from the realtime callback.

The bounded early seam is a non-realtime source-audio cache:

- decode source WAV fixtures before callback use
- store normalized interleaved `f32` samples with explicit sample rate and channel count
- compute RBX-285 W-30 per-bar hook evidence only during non-realtime source
  ingest after the final trusted/manual bar grid is known; the frozen analyzer
  receives decoded PCM and timing spans, never path, filename, or source-id
- expose bounded sample-window access for source-backed W-30 preview paths
- project a small fixed-size preview window from `CaptureRef.source_window` into callback-safe W-30 preview state
- write committed source-window captures as PCM16 WAV artifacts outside the realtime callback when the app has a session path and decoded source cache

- prefer loaded committed capture artifacts for focused W-30 pad playback / trigger state, falling back to source-window projection only for bounded audition surfaces when no artifact is available
- keep the `2048`-sample mono preview window for bounded audition diagnostics
- project focused committed pad artifacts into a separate callback-safe `16384`-sample mono representation that spans the full capture duration and carries original sample-rate / frame-count identity, playback rate, direction, loop crossfade, and a bounded source-derived chop plan
- project a focused lineage-ready resample artifact into a callback-safe
  `4096`-sample mono original-PCM grain with source start-frame, sample-rate,
  and frame-count identity; select the strongest energy/transient window
  deterministically outside the callback instead of time-compressing the full
  capture, point-sampling it, or decoding in realtime
- derive chop slice starts outside the callback from quantized short-time energy rises in the real capture; realtime transport and pad triggers may only select and retrigger the prepared bounded plan
- preserve the action-derived damage transform and capture-artifact identity across Session replay; artifact hydration must not invent macro / grit state that the committed action did not set
- apply committed W-30 articulation only after the ordinary source-backed W-30
  render and before bus mix. `pitch_dive_v1` preserves its first eight relative
  beats sample-exactly, then reads the existing rendered W-30 history causally
  for four beats with frozen rate `0.35 ^ progress`, fades over the final
  `0.15` beat, and emits silence from relative beat twelve. Its fixed-capacity
  history is allocated during callback-state preparation; overflow or missing
  source material fails silent. The action does not alter capture lineage,
  grit, Source Monitor, or the existing Hook Turnaround profile
- apply `filter_slam_v1` to the ordinary source-backed W-30 control sample
  before bus mix with one Direct Form II Transposed RBJ low-pass state per
  output channel. Update coefficients once per audio frame from the runtime
  sample rate. Across relative beats `[0,4)`, move cutoff exponentially from
  14 kHz to 1.8 kHz while Q moves from `0.707` to `0.85`; across `[4,6)`, move
  to 280 Hz/Q `1.2`; hold through beat seven; then crossfade filtered to dry
  over exactly 20 ms and preserve ordinary W-30 sample-exactly thereafter.
  Each moving segment uses `p²(3-2p)`. The action does not alter source PCM,
  playback rate, gate/reverse, grit, bus level, Source Monitor, or other lanes
- keep cache loading and source-window projection outside the realtime callback
- original-source file reads share `source_audio::read_source_wav_bytes` in Audio:
  admit a regular opened descriptor before reading; on Unix use nonblocking open
  so FIFO inputs fail without waiting for a producer. Original-source symlinks
  to regular files remain supported, unlike the stricter capture/export
  no-follow policies. App restore and live ingest hash the same returned bytes
  they decode; live ingest rejects a provider identity mismatch before Rust
  timing analysis or persistence (Source Graph RBX-429).
  Restore admission failures stay visibly unavailable with no source cache.
  The opened descriptor byte budget below is shared with capture hydration, not its path
  or no-follow policy. Regular-file I/O has no deadline guarantee.

#### Rust WAV/PCM resource admission V1 (RBX-425 / RIOTBOX-1556)

The non-realtime Rust WAV reader admits at most `268435456` encoded bytes
(256 MiB), including headers and ignored chunks. The PCM16/24 decoder admits
at most `67108864` interleaved samples, or 256 MiB of `f32` payload; this is a
sample count, not a frame count. These fixed limits are independent of source
results and have no public override. A new policy version/Decision is required
to change them.

- Check the opened regular descriptor's size before allocating payload; then
  bound actual stream reads even when metadata understates a growing file.
  The one-byte overrun probe stays in stack scratch. Never return a truncated
  successful WAV or reopen a path between admission, hashing and decoding.
- Use fallible, capped requested buffer growth and fallible exact PCM
  reservation before writing samples. `ResourceLimitExceeded` names encoded
  bytes or decoded samples with the limit and observed minimum required count;
  `AllocationFailed` names the failed resource. Allocator overhead/rounding is
  not the payload budget and successful reservation is not an OOM guarantee.
- `SourceAudioCache::from_pcm_wav_bytes` applies both limits to supplied bytes;
  caller-owned input allocation has already occurred. `from_interleaved_samples`
  adopts an already-owned vector and is outside this per-read/per-decode rule.
- Capture hydration keeps its own no-follow admission but shares the bounded
  opened-descriptor read and decoder. Original-source symlinks to regular files
  remain supported. Rejection installs no cache/trusted playback for that asset
  and preserves stored identity; independent valid captures may remain loaded.
- Existing larger assets that previously decoded become visibly unavailable,
  not silently truncated, rehashed/adopted, migrated or replaced with music.
  No Session schema, timing/analysis algorithm, DSP or realtime path changes.

This bounds individual Rust encoded/decoded payload requests only. Both
buffers may coexist, and multiple cached assets accumulate; it is not a global
cache/process memory cap, source-import memory guarantee, I/O/CPU deadline or
device-endurance proof. In particular, ingest invokes the Python sidecar before
Rust timing enrichment; its earlier file/frame/Python-sample allocations are
not bounded by this Rust contract. Capture/WAV writers and unrelated render
allocations are also outside it. Generated small-budget and sparse-file tests
do not qualify real sources, Holdouts or musical/release quality.

RBX-431 / RIOTBOX-1561 separately applies the same 256 MiB encoded-file budget
to the earlier Python provider read; Source Graph §6 owns that extension.
Python decoded/frame/analysis allocations and aggregate memory remain outside
both encoded-read policies. The Rust V1 reader/decoder contract above is unchanged.

RBX-433 / RIOTBOX-1563 additionally applies the same 67108864 interleaved-input-
sample count to Python before frame materialization, conversion and features.
Source Graph §6 owns that separate early admission. It is not a Python object/
heap bound or a reinterpretation of Python storage as Rust's `f32` payload.

The existing transport-selected source window remains the default. Only a
committed `feral_break_alpha_v2` preset with a non-baseline typed hook policy may
replace a one-bar `CaptureBarGroup` window, and only when persisted eligible
Source Graph evidence clears the frozen score-lift gate. Failure or insufficient
evidence retains the baseline with a typed lineage reason; it never synthesizes
fallback audio.

Current limitation:

- the initial cache supports PCM 16-bit and PCM 24-bit WAV fixture input
- input headers must have a nonzero frame block alignment representable by the WAV `u16` field; malformed headers fail during non-realtime ingest
- committed source-backed capture artifacts are PCM16 WAV files for the first app path
- raw audition without a committed artifact still uses a bounded preview excerpt
- focused committed pad playback is duration-aware and source-backed, but remains one mono pad seam rather than a full multi-pad streaming sampler engine
- the callback representation is a bounded full-duration proxy, not unbounded file streaming; broader codec support, stereo pad playback, multi-pad polyphony, and a full bank engine remain separate implementation slices

### 11.2 Bounded W-30 internal bus print seam

The first real W-30 internal resample print should be an offline app/control-plane operation, not a realtime recorder.

Smallest acceptable seam:

- input: one committed W-30 capture selected by the current lane focus
- source audio: prefer the committed capture artifact, then fall back to source-window projection only when no artifact exists
- render policy: use the existing W-30 preview / resample-tap state as the audible processing policy, including hydrated source audio, transport tempo/position, music-bus level, grit, source profile, and generation depth; no oscillator or synthetic fallback voice may substitute for missing capture audio
- duration: bounded to the input capture duration or a documented maximum window for MVP safety
- output: a new PCM16 WAV artifact at the derived resample capture `storage_path`
- session result: a new `CaptureRef` with `CaptureType::Resample`, explicit `lineage_capture_refs`, incremented `resample_generation_depth`, and no direct `source_window` unless the printed artifact is still a literal source-window copy
- Feral policy cue: when the loaded source graph has Feral break-support evidence and the resample keeps explicit lineage, the committed action result and capture note should expose this as a lineage-safe W-30 Feral rebake / reuse decision; if the same graph carries high quote-risk evidence, the cue must be held rather than approved
- realtime rule: the audio callback must never write files or perform offline bus prints

Minimum QA gate:

- control-path test for queue -> commit -> new resample capture -> lane focus
- artifact test proving the printed WAV exists, reloads, has expected sample rate / channel count / bounded duration, and is not silent
- comparison against raw source/capture artifact and missing-source silence,
  plus same-source determinism and cross-source diversity so the bus print
  cannot silently collapse to a dry copy or behave like the retired
  source-independent proxy voice
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
- W-30 preview retains its complete snapshot and publication revision in one
  callback-local cache. An unchanged even revision borrows that payload without
  rereading or copying sample arrays. Changed revisions use the same bounded
  stable-read check; rejected reads must not advance the cached revision.
  Before the first complete read the fallback is silent. Callback-owned timing
  is refreshed separately on every invocation, even when payload data is reused.
- source-monitor and live-master capture use RBX-427's preallocated borrowed
  SPSC publication rather than first-use thread-local snapshot registration.
  Their unique noncloneable readers are acquired during control-side processor
  preparation; a repeated acquisition fails closed there. Neither reads nor
  slot refreshes may allocate, reclaim payloads, or acquire control mutexes.
  Control ownership must outlive stream shutdown. Processor/worker teardown is
  distinct from a callback invocation and is not claimed heap-free.
- test-only per-thread heap accounting must exercise the exact production
  processor, including a first invocation in a fresh process/worker without
  callback-side warmup. Cover allocation, zeroed allocation, reallocation and
  deallocation, with positive controls; synthetic checks do not prove host
  deadlines, device endurance or human audio quality.

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

- the shared sample-peak knee `0.92` and ceiling `0.985` are retained as the
  accepted bounded engineering baseline (RBX-421 / RIOTBOX-1501), replacing
  RBX-379's pending retention decision without changing their values or DSP.
  Magnitudes at or below the knee pass unchanged; strictly greater magnitudes
  enter the existing tanh knee capped at the ceiling. Retaining these values is
  not a claim of true-peak, device or hearing-safety calibration. Exact clean-
  path QA still requires zero limiter-modified samples and zero pre/post clips.
  The [retention decision](../reviews/riotbox_1501_limiter_retention_decision_2026-10-02.md)
  binds synthetic controls, three Development cases and the exact Sparse human
  comparison: no audible policy distinction or preference. It establishes no
  perceptual optimum, universal source equivalence or musical-quality pass;
  absence of a demonstrated alternative benefit justifies no production change
- [Source-free baseline protocol v1](../benchmarks/master_bus_limiter_baseline_protocol_v1.md)
  (RIOTBOX-1550 / RBX-417) fixes in-memory boundary/overload/partition and named
  five-owner RuntimeMix controls. Actual writes/counting require a shaped delta
  strictly greater than `f32::EPSILON`; mathematical above-knee count is not the
  modification count. This is a baseline prerequisite, not RIOTBOX-1501
  calibration or authorization for source, alternative policy or playback.
- The default-off `limiter-calibration` diagnostic feature reuses that exact
  limiter implementation for fixed offline A/B/C comparisons and retains the
  existing offline RuntimeMix pre-buffer. It adds no live callback sink or
  production policy knob. Execution, source admission and interpretation belong
  to the [bounded Development v1 protocol](../benchmarks/master_bus_limiter_calibration_protocol_v1.md)
  and its separately authorized [fixed-overload v2 follow-up](../benchmarks/master_bus_limiter_calibration_protocol_v2.md);
  an available diagnostic interface is not itself an execution grant or a
  completed calibration decision.
- product runtime mixes and Feral-grid product mixes pass through the shared
  master-bus soft-limiter seam after source-monitor / lane mix policy and
  before device or WAV output
- Source Monitor `blend` exposes its unclamped generated-plus-source sum to
  that master seam; it must not hard-clamp inside the monitor policy, because
  doing so flattens transients and hides the pre-limiter overload evidence
- the limiter is stateless and in-place so realtime rendering does not allocate,
  block, log, or call analysis/model code
- reports keep pre-limiter and post-limiter metrics for controlled mixes so
  `clip_count > 0`, weak RMS, source-character collapse, or fallback-like output
  remain visible instead of being hidden by output-file clipping
- the limiter may reduce hot transient peaks but must not boost weak, silent, or
  source-characterless output; those remain QA failures
- runtime-master capture observes the post-limiter samples immediately before
  device conversion. V1 preallocates at most `16,777,216` interleaved atomic
  sample slots on the control thread, never spins while retiring callback-owned
  buffers, and stops the output stream before control-thread WAV/proof/Session
  I/O begins
- capture publication uses the same borrowed SPSC policy. Retained slot Arcs
  are ownership, not active callback usage. A per-buffer permanently closable
  admission gate and explicit in-flight borrowed leases protect all callback
  timing/sample work. RBX-428 encodes the closed bit and lease count in one
  atomic word: registration increments and checks its returned previous word,
  closure sets the bit, and release decrements after work. These operations are
  SeqCst with a half-range overflow-abort bound and no retry/spin loop.
  Finish preserves the completeness precheck, then closes admission
  before checking quiescence and detaching; active work returns the existing
  nonwaiting `CallbackStillActive` outcome and remains control-retained.
  Abort closes before detaching. Stale readers cannot write to a closed buffer,
  and a new begin never reopens it. Test the actual admission module with Loom;
  reference counts alone cannot establish safe finalization.
- V2 reuses that allocation and tap but arms until an absolute Session beat.
  Pre-boundary callbacks write no payload; a straddling callback begins at the
  first complete frame at or after the requested beat. The callback publishes
  actual start/end positions and fails closed on a missed boundary or any
  discontinuity between captured callback windows. Callback-gap telemetry runs
  across both the armed wait and copied window. It still allocates nothing,
  takes no lock, and performs no I/O, analysis, or Session mutation
- V3 (RBX-422) admits explicitly selected eight- or sixteen-bar windows through
  that same generic frame-count tap, not a new recorder or DSP path. Core owns
  the typed duration and checked canonical-runtime-BPM frame calculation;
  micro-BPM evidence must reconstruct the original f32 tempo exactly. App enforces
  the unchanged `16,777,216` interleaved-slot ceiling before arming. Longer
  windows do not raise allocation limits, loosen health/physical frame gates or
  add callback work. Historical V1/V2 retain their meaning; RBX-423 separately
  selects canonical V4 for future default two-bar recordings.
  Generated callback-partition and publication tests establish implementation
  evidence only; extended real-device and human qualification remain separate.

V3/V4 recording position arithmetic (RBX-422, extended to exactly two bars by
RBX-423): the callback clock accumulates
positive f64 beat spans. An exact long sample window can therefore exceed V2's
fixed `1e-6`-frame endpoint comparison without a skipped sample or transport
fault. Keep V2 unchanged. V3 checks physical duration from the exact frame count
times the canonical runtime beats-per-frame, retaining the `0.500001`-frame
rounding limit and the existing start-alignment gate. Retain actual callback
start/end positions in evidence, never replace them with idealized endpoints.

For canonical V3/V4 only, compare the actual end with `start + frames * beats_per_frame`
using `max(beats_per_frame * 1e-6, gamma(2K+16) * M)`, where `K` is the captured
callback count, `M=max(abs(start),abs(end),abs(expected_end),1)`,
`gamma(n)=n*u/(1-n*u)` and `u=f64::EPSILON/2`. The two operations per callback
cover span multiplication and accumulated addition; sixteen extra operations
conservatively cover endpoint interpolation and comparison arithmetic. Require
`1 <= K <= frames`, finite positive geometry, and `n*u < 1`. Reject if the bound
exceeds either `1/1024` frame or `1e-6` beat, the existing serialized microbeat
budget. This is a bounded representation guard, not permission for a missing
frame, discontinuous clock or degraded callback health. Core's existing
microbeat readiness checks remain mandatory. V3 proof `duration_error_frame_micros`
records nominal frame rounding, while actual endpoint discrepancy is checked
against this arithmetic bound before publication. No realtime clock/DSP change
or new host/listening claim follows. V4 changes the future default two-bar
producer to this canonical contract while leaving historical V2 arithmetic
and evidence unchanged. The callback clock/tap and allocation ceiling stay
unchanged; synthetic tests do not transfer existing device/listening evidence
to a fresh V4 qualification.
- the Feral-grid development product-stem seam attributes the already rendered
  generated-support mix across typed TR-909 drums, W-30 music, and MC-202 bass
  contributions after the shared nonlinear product bus. The symmetric
  three-lane Shapley allocation changes no full-mix sample; the written PCM16
  contribution stems must reproduce that written mix through direct sample
  summation with maximum absolute error at most `3 / 32768` and RMS error at
  most `1.5 / 32768`
- product-stem attribution and WAV/proof validation are deterministic offline
  work only. They do not run on the realtime callback or claim isolated pre-bus
  lane audio. The MC-202 contribution may use `source_derived` origin only after
  the already rendered source-expression plan, source-contour delta,
  bass-pressure proof, source-grid alignment, and no-fallback evidence all pass;
  otherwise publication stops fail-closed
- the bounded source-matched stem-package consumer performs proof parsing,
  hashing, PCM16/grid decoding, reconstruction measurement, copying, output
  measurement, JSON writing, and observer emission on the app/control path. It
  copies the already-proven contribution bytes and adds no DSP, mixing,
  fallback music, or realtime callback work

Offline and realtime-simulation renders should become comparable under the same
state, with explicit tolerances where backend buffer boundaries or floating
point differences make bit identity unrealistic.

Current parity seam:

- `RuntimeMixRenderPlan` captures the transport plus TR-909, MC-202, W-30
  preview, W-30 resample, and Source Monitor render state needed by the runtime
  mix
- `render_runtime_mix_offline` renders that plan as one deterministic offline
  block through the same mix and Source Monitor policy functions used by the
  callback
- `render_runtime_mix_realtime_simulation_offline` renders the same plan in
  bounded callback-sized blocks, advancing transport timing between blocks
- `render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report`
  retains callback state across plan steps and returns the audible samples plus
  exact aggregate pre-limiter metrics, post-limiter metrics, and the count of
  samples changed by the limiter; this reporting allocation is offline-only and
  does not change the live callback's allocation-free contract
- parity tests compare the full-block render against the callback-blocked
  simulation, so callback buffer boundaries cannot silently change runtime mix
  audio
- exact-mixer diagnostic packs gate pre-limiter clipping, limiter activity, and
  post-limiter clipping separately. A clean-path proof must not pass merely
  because the master limiter hid a hot product mix; its maximum accepted
  limited-sample count is explicit in the manifest

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

Runtime stream-error diagnostics are owned by `runtime/telemetry.rs` (RBX-380).
The render callback writes only the existing atomic counters/timing. Device
stream-error reporting and full health snapshots may lock the diagnostic message
slot; they are not the normal render callback. A poisoned slot is recovered for
reading/replacement without clearing poison or fabricating a device-error count.
Public health keeps a degraded diagnostic reason (including the last reported
message when present); a running runtime is reported as `Faulted`. Stopped
lifecycle remains stopped. Creating a new runtime creates fresh telemetry.
This is bounded poison recovery, not a claim that full health snapshots are
lock-free or that an arbitrary runtime panic is recoverable. Timing-only
snapshots read atomics directly and do not lock or clone diagnostic messages.

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

### Bounded Output Probe Observation V1

RBX-435 / RIOTBOX-1565 makes the existing source-free `cpal_spike` diagnostic
fail closed for automation. `Ok` requires Running or Stopped health, at least
one observed output callback, zero stream-error/scratch-overflow counters and
no retained stream-error detail. Failure precedence is stream error/detail,
scratch overflow, lifecycle, then missing callback observation. The summary
preserves both error counters and context; process exit is zero only for typed
`Ok`, nonzero for `Failed` or `NotRun`.

The silent defaults, fixed 250 ms duration and snapshot-before-stop lifetime
are unchanged. Zero callbacks means observation was not established in that
window, not proof of a broken backend. Gap metrics have no success threshold;
this is not latency, xrun, continuous-health, route-isolation, audibility or
endurance qualification. Tag actual observations as sandbox or real session;
source-free unit/reporting tests never open a device. Historical observations
are not upgraded by this contract, and RIOTBOX-1041 still owns real-session soak.

RIOTBOX-1566 / RBX-436 adds a separate dev-only
[silent virtual-host observation](../benchmarks/silent_host_observation_v1.md).
Its fixed driver reuses this health projection and the default idle runtime;
the Linux operator owns backend/route containment, bounded supervision and
ephemeral evidence. Generated tests never open CPAL. Even a successful attempt
is only sampled silent virtual-endpoint evidence, not completion of the soak,
loaded-runtime, physical-device, musical or release gates above.

V1's actual attempt is consumed and failed. RIOTBOX-1567 / RBX-437 repairs
[Node lifetime and Pulse module identity in V2](../benchmarks/silent_host_observation_v2.md)
without changing the Rust driver, runtime or any operational budget. This is
generated-only verification; the normal host-launch entry point is disabled.
The original evidence is not upgraded, and another host attempt requires a
separate prospective phase and owner.

RIOTBOX-1570's separately authorized V2 call failed admission on native multiline
module metadata before creating a sink or starting CPAL. Its evidence and
authorization are consumed. RIOTBOX-1571 / RBX-440 defines the source-free
[V3 metadata repair](../benchmarks/silent_host_observation_v3.md), preserving
the driver/runtime, numeric values and route/process protections. Paired JSON
payload/text-index framing, opaque native argument handling and exact named-Node
collision admission must not enable the CLI or imply another host attempt.

RIOTBOX-1572's separately authorized V3 observation admitted the owned virtual
route and completed one silent 60-second run; the second recorded one backend
buffer underrun/overrun at sample 36 and stopped. Cleanup was verified, the
third run did not start, and the full observation gate failed. The consumed
negative result does not change V3/runtime thresholds or qualify endurance.

RIOTBOX-1574 / RBX-441 adds a separately authorized
[V4 ALSA-boundary observation](../benchmarks/silent_host_observation_v4.md).
It preserves the driver, runtime, route protections and numeric budgets while
adding a QA-only, child-bound measurement leaf. Generated regression tests
prove error attribution and diagnostic fail-closed behavior, not the original
backend trigger. Instrumented plugin geometry and timing do not establish
physical latency, musical quality, a root-cause fix or completion of RIOTBOX-1041.
The separately approved V4 observation completed all three silent runs without
reported or measured ALSA errors; cleanup was separately verified. Its consumed
[observation record](../reviews/riotbox_1574_alsa_diagnostics_2026-10-09.md)
does not explain or erase the earlier V3 failure, and does not grant another run.

---

## 16. Open Follow-Ups

This draft should be followed by:

1. backend selection decision
2. transport implementation decision
3. exact bus graph layout
4. capture buffer and file-write policy
