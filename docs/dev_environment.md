# Riotbox Development Environment Notes

Status: active workflow note
Audience: contributors and coding agents

This document holds environment and sandbox details that are useful when needed
but too operational for the always-loaded `AGENTS.md` brief.

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
- Keep `just ci` as the final PR gate. RIOTBOX-1399 owns a measured scoped inner
  loop when validation cost blocks audible delivery.

For new audible work, follow the
[P023 audible-delivery plan](./plans/p023_audible_delivery_course_correction.md):
focused seam tests and one exact render/preflight during exploration, applicable
product/source gates during qualification, and the broad workflow gate before
merge.
