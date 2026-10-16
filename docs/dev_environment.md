# Riotbox Development Environment Notes

Status: active workflow note
Audience: contributors and coding agents

This document holds environment and sandbox details that are useful when needed
but too operational for the always-loaded `AGENTS.md` brief.

## Session Storage

External Session/Source Graph saves require a target filesystem that supports
hard links for immutable generation publication (RBX-374). Unsupported
publication fails explicitly before replacing the alias or Session; use a
supported local filesystem or the existing embedded-Graph mode. There is no
copy fallback. This is a storage requirement, not a claim of power-loss
durability or concurrent-writer support. Synthetic failure injection covers the
error path; a real filesystem matrix has not been qualified.

## Memory And Search

- Riotbox currently has no active semantic-memory tool in the development
  workflow.
- Canonical truth lives in `docs/`, `plan/`, Linear, and Git history.
- Use `rg`, `just decision-search "query"`, targeted file reads, and Linear /
  GitHub lookup for normal work.
- `just decision-search "query"` is a bounded `rg` helper over
  `docs/research_decision_log.md`; it does not use any
  semantic index.
- Do not store canonical decisions only in chat, local memory, or future
  semantic-memory tools.

## Audio And Device Probing

- Do not assume a failed audio probe inside the sandbox means the machine audio stack is broken.
- Record whether Linux audio validation came from sandbox or real user session.
- Treat sandbox-only audio failures as inconclusive.

The dev-only [RIOTBOX-1566 V1 silent-host attempt](benchmarks/silent_host_observation_v1.md)
is consumed and failed; it must not be repeated. The
[RIOTBOX-1567 V2 repair](benchmarks/silent_host_observation_v2.md) changes only
lifetime/module identity checks and is source-free. The operator CLI rejects
execution even with the old explicit flag. `just silent-host-contract-fixtures`
uses generated metadata/children, never the host audio service. The separately
authorized [RIOTBOX-1570 V2 invocation](reviews/riotbox_1570_silent_host_v2_observation_2026-10-09.md)
is also consumed and failed before any audio start. The
[RIOTBOX-1571 V3 metadata repair](benchmarks/silent_host_observation_v3.md)
uses generated fixtures only; it grants no retry. A future host phase needs
fresh explicit authorization and its own prospective owner/revision/binary
binding, and must retain exclusive PipeWire-only ALSA configuration, not the
user's potentially physical default.

The separately authorized [RIOTBOX-1572 V3 observation](reviews/riotbox_1572_silent_host_v3_observation_2026-10-09.md)
is consumed: preflight and run 1 completed, run 2 reported a backend buffer
underrun/overrun and stopped; run 3 did not start. Owned cleanup and unchanged
default/mute/volume were verified. This is negative real-session evidence, not
an endurance pass or permission to retry. The frozen V3 operator stays unchanged.

[RIOTBOX-1574 V4](benchmarks/silent_host_observation_v4.md) adds an opt-in,
QA-only ALSA measurement adapter to that operator. Its separately authorized
one-shot fence pins the diagnostic library as well as the reviewed Rust binaries
and protocol. Only the target audio child is instrumented, not timeout/metadata;
read diagnostics after verified group cleanup. It changes no production runtime,
buffer, priority or original verdict and does not enable the ordinary CLI.
An instrumented observation is not a zero-overhead or original-trigger fix claim.

The separately authorized [RIOTBOX-1579 loaded V1 observation](reviews/riotbox_1579_loaded_recorder_observation_2026-10-16.md)
is consumed and failed after its first two-bar stopped-window marker: the
observer rejected terminal reuse of an audio Node's global ID by a non-audio
Client. No finished recording pass exists; eight/sixteen-bar cases did not run.
Owned cleanup and unchanged default/mute/volume were verified. The generated
fixture, stopped milestone and zero-error partial ALSA measurements do not grant
musical, endurance or physical-device qualification. RIOTBOX-1580 owns the
source-free observer correction; it authorizes no retry or new host invocation.

The [loaded-observer V2 correction](benchmarks/loaded_recorder_observation_v2.md)
is source-free only and explicitly selected. It distinguishes a validated
non-audio Client reusing a released Node ID from a replaced audio Node, while
preserving all route, completion, deadline, artifact and cleanup gates. V1
remains the historical default and retains its original rejection. No ordinary
execution CLI or new actual observation is enabled by this version.

The [RIOTBOX-1581 prospective loaded V2 phase](benchmarks/loaded_recorder_host_riotbox_1581_v2.md)
is separate preparation, not an execution grant. Its explicit V2 fence requires
fresh phase-specific approval and exact reviewed local/native CI, binary,
protocol and fresh generated-fixture bindings. The consumed 1579 approval cannot
substitute. No actual result exists until that new fence is independently admitted
and invoked once; first failure stops with no retry.

The [actual RIOTBOX-1581 V2 observation](reviews/riotbox_1581_loaded_recorder_v2_observation_2026-10-16.md)
is now consumed: its separately approved 250-ms preflight and generated
2/8/16-bar recordings passed actual artifact/Session/receipt/Core/route/diagnostic
gates with verified cleanup and unchanged defaults. This is only instrumented
isolated generated-composite recording evidence, not musical, physical-device,
latency, endurance, release or original ALSA-cause qualification. No further
trial is authorized and the failed V1 result remains unchanged.

The [RIOTBOX-1589 post-repair phase](benchmarks/loaded_recorder_host_riotbox_1589_v1.md)
prospectively reuses that unchanged generated V2 observation for the now-repaired
callback/control and history-capacity paths. Its new owner, fresh fixture and
exact-revision bindings require separate explicit approval; preparation is not
host access. The 1581 result and consumed permission stay unchanged. This is a
current-runtime regression, not longer endurance, musical or physical-device
qualification. No actual 1589 result or further trial is implied.

## Agent Sandbox Self-Checks

Run these when Riotbox runs inside `agent-sandbox` and host capability is unclear:

```bash
command -v git
command -v cargo
command -v pkg-config
pkg-config --libs --cflags alsa
```

Interpretation:

- If `pkg-config --libs --cflags alsa` fails, the sandbox cannot build the current Linux audio path cleanly.
- `just` is convenient but not required; prefer direct script commands if `just` is absent.

Preferred solution:

- Bake needed tooling into the sandbox image.
- Use mounts only for host-specific assets or sockets.

## Arch Host Requirements

Source-free DAWproject schema tests require `xmllint` (`libxml2` on Arch,
`libxml2-utils` on Ubuntu CI). They validate synthetic XML against repository-
pinned upstream schemas using `--nonet`; they never fetch schemas at test time.
This is a development/CI requirement, not an audio-runtime dependency.

Audio build requirements:

- `pkg-config` available in the sandbox.
- ALSA headers and pkg-config data visible in the sandbox.
- `PKG_CONFIG_PATH=/usr/lib/pkgconfig`.

Useful Arch host mounts:

- `/usr/include/alsa` -> `/usr/include/alsa`
- `/usr/lib/pkgconfig` -> `/usr/lib/pkgconfig`
- `/usr/lib/libasound.so` -> `/usr/lib/libasound.so`
- `/usr/lib/libasound.so.2` -> `/usr/lib/libasound.so.2`

## Git Push Ergonomics

- Ensure SSH auth is available.
- Ensure GitHub host trust is available.
- A temporary `known_hosts` file is a workaround.
- Better sandbox setup: writable `~/.ssh/known_hosts` or pre-seeded GitHub host keys.

## Host Services

- Use `host.containers.internal` for host-local TCP services.
- Do not assume `localhost` means the host. In the sandbox it is container-local.

## Current Course-Correction Environment Boundary

The dated RIOTBOX-1439 capability and cost measurements live in
`docs/reviews/riotbox_1439_delivery_system_audit_2026-08-16.md`. The durable
environment rules are:

- Rust/Cargo, `just`, FFmpeg/ffprobe, `jq`, ALSA development metadata,
  PipeWire playback, GitHub auth, and Linear access are expected for the full
  local workflow; recheck them when the execution context changes.
- `.env.local` stays ignored. Never print or copy its secret values into logs,
  issues, artifacts, or commits, and keep environment/credential files untracked.
- Select `just` recipes from the active contract; recipe count is not evidence
  quality.
- Local `target/` and `artifacts/` trees are reusable state. Never clean either
  automatically; inspect exact targets and ask before material deletion.
- Use source-free `just ci` as the normal final PR gate. Reserve the guarded
  `RIOTBOX_BROAD_AUDIO_QA_ACCESS=registered-development-only just ci-broad` for
  an explicitly authorized phase/release regression; the acknowledgement never
  authorizes Holdout audio, commercial references, or source discovery.

For new audible work, follow the
[P023 audible-delivery plan](./plans/p023_audible_delivery_course_correction.md):
focused seam tests and one exact render/preflight during exploration, applicable
product/source gates during qualification, the source-free PR gate before
merge, and the guarded broad gate only at an authorized phase/release boundary.
