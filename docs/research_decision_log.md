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
