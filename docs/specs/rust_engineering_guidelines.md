# Riotbox Rust Engineering Guidelines

Version: 0.1  
Status: Draft  
Audience: contributors working in the Rust codebase

---

## 1. Purpose

This document captures the Rust-specific working rules for Riotbox.

It exists so the codebase stays:

- explicit
- testable
- replay-friendly
- ready for later realtime work

---

## 2. Core Principle

Prefer simple, explicit Rust over clever Rust.

For this project, the main risk is architectural drift and hidden state, not a lack of advanced language tricks.

---

## 3. Modeling Rules

- Prefer `struct` and `enum` types over stringly behavior.
- Keep domain types small and named after the product contracts.
- If a value affects replay, restore, or capture lineage, model it explicitly.
- Avoid hidden global state.
- Keep ownership boundaries visible in the type design.

---

## 4. Realtime-Oriented Rules

- Keep blocking work out of future audio-path code.
- Avoid APIs that make ownership, timing, or mutation obscure.
- Prefer bounded queues and explicit handoff points.
- Treat allocations and cloning in future callback-path code as suspicious until proven harmless.

These rules matter even before the audio callback exists, because early model choices shape later runtime safety.

---

## 5. Error Handling

- Use typed errors at subsystem boundaries.
- Do not use panic-driven control flow for expected failures.
- Surface degraded states explicitly.
- Prefer honest failure over silent fallback that corrupts state semantics.

---

## 6. Dependencies

- Add dependencies conservatively.
- Prefer the standard library until a real need appears.
- Do not add convenience crates just to save a few lines during early model stabilization.
- New dependencies should have a clear owner and reason.

Current expected direction:

- `serde` / `serde_json` for persistence
- `tokio` for control-plane async work
- `cpal` for audio I/O
- `ratatui` for terminal UI

---

## 7. Testing

- Keep unit tests close to the module they validate.
- Test product contracts, not implementation accidents.
- For early slices, prioritize:
  - data model sanity
  - queue behavior
  - replay-related invariants
  - view-model correctness

Minimum commands:

```bash
cargo fmt
cargo test
```

Preferred additional checks once dependencies grow:

```bash
cargo check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 8. Module Design

- Keep modules small and responsibility-focused.
- Do not bury unrelated concepts in one giant file once the codebase grows.
- Keep app orchestration separate from core domain models.
- Keep sidecar-facing contracts explicit at crate boundaries.

---

## 9. Serialization Guidance

- Favor human-readable formats while contracts are still moving.
- Derive serialization only when the schema is clear enough to test.
- Treat serialized shape as a product contract, not as an incidental implementation detail.

---

## 10. Documentation Discipline

- When a code change freezes or changes behavior, update the matching spec.
- If a technical decision becomes stable, record it in `docs/research_decision_log.md`.
- Do not let code become the only place where architecture is explained.

---

## 11. Repository Workflow

- Use `cargo` as the primary build and test interface.
- Use a small task runner only as a convenience layer, not as a replacement for understanding the underlying Cargo commands.
- Keep root-level automation shallow and readable.

---

## 12. Current Recommendation

For Riotbox today:

- `cargo` is the native standard
- a small root `Justfile` is useful for convenience
- a `Makefile` is not necessary unless the repo later needs broader polyglot or packaging workflows
