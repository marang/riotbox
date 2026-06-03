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
