# Riotbox Technology Stack Spec

Version: 0.1  
Status: Stack Freeze v1  
Audience: whole project

Derived from:
- `docs/execution_roadmap.md`
- `docs/specs/audio_core_spec.md`
- `docs/specs/session_file_spec.md`
- `docs/specs/tui_screen_spec.md`

---

## 1. Purpose

This document freezes the minimum technology decisions required to start the first implementation slices without reopening architecture debates every day.

It is deliberately narrow.

It fixes only the stack decisions that block:

- Source Graph v1
- Session v1
- Action types and queue
- Jam view model
- the first core runtime slice after that

---

## 2. Freeze Rule

`Stack Freeze v1` should remain stable until the first core skeleton and analysis vertical slice are both real.

Changes are allowed only if:

- a spike disproves the decision
- a benchmark makes the choice untenable
- the implementation cost becomes clearly worse than switching

If a stack choice changes, the reason must be logged in `docs/research_decision_log.md`.

---

## 3. Frozen Stack v1

### 3.1 Core language

- `Rust`

Why:

- best fit for realtime audio, deterministic state, action scheduling, and explicit ownership boundaries
- more appropriate than Go for callback-sensitive audio work

### 3.2 Analysis sidecar language

- `Python`

Why:

- fastest path for MIR / ML / offline analysis tooling
- broad ecosystem for analysis experiments and provider bakeoffs
- easy process boundary from the core

Clarification:

- this is the current recommended implementation language for the sidecar, not an irreversible architecture commitment
- the sidecar boundary should remain language-neutral enough that later implementations can move parts of the sidecar to `Rust` or another language if the contract remains stable
- Source Timing Intelligence is a Rust-first product contract. Python or external MIR tooling may be used for research comparison, but the durable timing model, replay surface, Source Graph schema, QA gates, and runtime consumers must not depend on a Python-only implementation.

### 3.3 Core repository shape

- Rust workspace at repo root
- initial crate: `crates/riotbox-core`
- later likely additions:
  - `crates/riotbox-app`
  - `crates/riotbox-audio`
  - `python/sidecar`

### 3.4 Serialization direction

- human-readable `JSON` for sessions and graph-adjacent persisted artifacts in v1

Why:

- easy inspection while contracts are still settling
- simple debugging and migration visibility

Clarification:

- this applies to persisted artifacts in v1, not to a permanently frozen RPC or sidecar transport choice
- sidecar transport encoding should be revisited once the request/response contract becomes more stable

### 3.5 TUI direction

- Rust TUI built around `ratatui`

Why:

- mature terminal-native Rust path
- good fit for the Jam-first interface

### 3.6 Async / control runtime direction

- `tokio` for non-realtime orchestration once the runtime layer arrives

Why:

- strong fit for sidecar orchestration, control plane tasks, and bounded async work

### 3.7 Audio backend direction

- Rust audio path based on `cpal`

Why:

- direct low-level fit for cross-platform audio I/O without overcommitting to a higher-level engine too early

### 3.8 Logging / observability direction

- structured logging and health metrics from the Rust core
- benchmark-visible health reporting from day one

### 3.9 Error handling direction

- explicit typed errors at subsystem boundaries
- no panic-driven control flow in the runtime path

---

## 4. Explicit Non-Choices

These are intentionally **not** frozen yet:

- final Python analysis library set
- final long-term sidecar implementation language
- final RPC transport format
- final sidecar wire encoding
- exact audio backend wrapper architecture
- exact persistence crate stack
- final CLI shape
- packaging and installers
- plugin architecture

Those should be decided by spikes, not by speculation.

Revisit triggers for the sidecar stack:

- more message types than the current narrow request/response spike
- progress events or streaming responses
- concurrent analysis jobs
- reconnect or external sidecar lifecycle requirements
- schema/versioning pressure strong enough to justify `Protobuf` or another stricter format
- evidence that parts of the sidecar should move out of `Python` for performance, determinism, or maintenance reasons

---

## 5. Why Not Go

Go is rejected for the main core because:

- garbage collection is the wrong default tradeoff for callback-sensitive audio work
- it is a weaker fit for DSP-leaning and hard timing-sensitive internals
- Riotbox needs explicit ownership and bounded state transitions more than service-style concurrency

Go may still be acceptable later for isolated tooling, but not as the primary Riotbox core language.

---

## 6. Immediate Spike List After Freeze

The next validating spikes should be:

1. audio latency spike in Rust
2. sidecar transport spike between Rust and Python
3. deterministic replay spike
4. session serialization spike

---

## 7. Implementation Consequence

Because of this freeze:

- the first real model layer will be implemented in Rust
- the current repo should begin as a Rust workspace
- sidecar code can wait until the transport and analysis spikes begin

---

## 8. Exit Condition For This Freeze

`Stack Freeze v1` has done its job when:

- the core model layer compiles
- the first queue and Jam view tests pass
- the first runtime slice can start without reopening language or framework arguments
