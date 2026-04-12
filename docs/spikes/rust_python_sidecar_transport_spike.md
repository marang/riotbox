# Rust-Python Sidecar Transport Spike

Version: 0.1  
Status: Draft  
Ticket: `RIOTBOX-9`

---

## Purpose

Validate one practical transport choice between the Rust core and a future Python analysis sidecar.

Questions:

- what message framing should we start with
- how should failure isolation work in the first sidecar slice
- can Rust request a Source Graph shaped response from Python without inventing a heavy RPC stack too early
- what should stay explicitly out of scope for the first transport contract

---

## Prototype

Repo prototype:

- Rust crate: `crates/riotbox-sidecar`
- Python script: `python/sidecar/json_stdio_sidecar.py`
- Probe binary: `cargo run -p riotbox-sidecar --bin stdio_probe`

What it does:

- spawns a Python sidecar as a child process
- exchanges newline-delimited JSON over `stdin` / `stdout`
- validates a `ping` roundtrip
- requests a stub Source Graph build
- deserializes the Python response directly into the existing Rust `SourceGraph` model

This is intentionally a transport spike, not the production analysis runtime.

---

## Chosen Candidate

Use newline-delimited JSON over `stdio` as the first transport contract.

Why this is the right v1 spike target:

- simplest failure boundary between Rust core and Python sidecar
- easiest message inspection during early contract stabilization
- no daemon, socket setup, or background service lifecycle required
- good fit for request/response analysis jobs where the sidecar may restart without taking audio down

---

## What This Spike Validates

- Rust can spawn and talk to a Python sidecar without adding async or socket complexity first
- newline-delimited JSON is sufficient for the first request/response contract
- existing Rust core models are serializable enough to accept a sidecar-produced `SourceGraph`
- sidecar failure remains process-local rather than contaminating core state directly

---

## What This Spike Does Not Yet Validate

- long-running concurrent jobs
- progress streaming
- multi-request multiplexing
- cancellation
- backpressure policy
- target analysis latency
- socket-based reconnect behavior
- final production RPC schema

---

## Alternative Candidates Considered

### Unix domain sockets

Pros:

- better fit for a long-lived sidecar process
- cleaner separation once concurrent requests and reconnect semantics matter

Why not first:

- more lifecycle and environment complexity than the current question requires
- weaker debugging ergonomics during early contract stabilization

### Localhost TCP

Pros:

- portable and familiar
- easy to inspect with external tools

Why not first:

- unnecessary network surface for a same-machine sidecar
- more setup and error cases than the current spike needs

### MessagePack or Protobuf

Pros:

- smaller payloads
- stronger schema discipline in later phases

Why not first:

- lower human readability during spec churn
- premature optimization before request shapes stabilize

---

## Local Observation

Observed with:

```bash
cargo test -p riotbox-sidecar
cargo run -p riotbox-sidecar --bin stdio_probe
```

Result:

- Rust tests verify NDJSON request and response roundtrips
- an end-to-end test successfully spawns the Python sidecar and receives:
  - `pong`
  - `source_graph_built`
- the probe binary prints a successful `pong` and a stub graph summary

Interpretation:

- the transport path is viable enough for early analysis integration work
- the contract should stay narrow and synchronous until real analysis workloads force more complexity

---

## Initial Recommendation

Use `stdio` plus newline-delimited JSON for the first real sidecar-facing slice.

Recommended boundaries:

- keep the Rust side responsible for process lifecycle, request IDs, and typed decode
- keep the Python side responsible for analysis execution and graph production
- keep audio and realtime state entirely outside this transport path

Future switch trigger:

- move to Unix sockets only when job concurrency, reconnect semantics, or external sidecar lifecycle make `stdio` too restrictive

---

## Follow-Up Work

1. define the first non-stub analysis request schema
2. add explicit sidecar health and restart reporting to app state
3. decide whether progress events need to be part of the protocol before the analysis vertical slice grows
4. keep request and response versioning explicit in every message
