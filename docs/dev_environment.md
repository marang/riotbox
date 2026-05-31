# Riotbox Development Environment Notes

Status: active workflow note
Audience: contributors and coding agents

This document holds environment and sandbox details that are useful when needed
but too operational for the always-loaded `AGENTS.md` brief.

## MemPalace

- MemPalace is optional dev memory.
- It is not product core.
- It is not a source of truth.
- Canonical truth lives in `docs/`, `plan/`, Linear, and Git history.
- Use MemPalace to complement `rg`, not replace it.
- Do not store new canonical decisions only in MemPalace.

Repo-local layout:

- `.mempalace/palace/` stores the persistent Chroma database.
- `.mempalace/cache/` stores model and package cache.
- `.mempalace/results/` stores captured evaluation outputs.
- `.mempalace/corpus/` stores copied project corpus for mining.

Operational path:

- Use `just mem-init` for first setup.
- Use `just mem-status` and `just mem-search "..."` for normal work.
- Use `just mem-repair` for index metadata drift such as missing cosine-distance metadata.
- The wrapper uses rootless Podman with pinned `python:3.14.4-slim` and `mempalace==3.3.4`.
- Normal runtime commands run with container networking disabled.
- Image builds require normal registry/network access.
- The wrapper re-mines when mined repo sources changed.
- The wrapper uses a repo-local lock to prevent concurrent mining.
- Do not hand-edit `.mempalace/corpus/mempalace.yaml`.
- The wrapper syncs selected live repo sources into room-specific folders.
- Active rooms: `specs`, `workflow`, `reviews`, `audio_qa`, `plan`, `decisions`, `code`, `documentation`, and `general`.
- If room structure changes, the wrapper rebuilds the palace index on the next mine/status/search.
- The wrapper rebuilds the MemPalace container image only when compose/container files change.

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
command -v podman
pkg-config --libs --cflags alsa
test -S "/run/user/$(id -u)/podman/podman.sock" && echo podman-socket-ok
```

Interpretation:

- If `pkg-config --libs --cflags alsa` fails, the sandbox cannot build the current Linux audio path cleanly.
- If `podman` is missing or the Podman socket is unavailable, MemPalace operational commands cannot run from inside the sandbox.
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

MemPalace operational requirements:

- `podman` client available in the sandbox.
- `podman compose` support available in the sandbox.

If using host rootless Podman instead of nested Podman:

- Mount `/run/user/<host-uid>/podman/podman.sock`.
- Expose it at the same path or a known sandbox path.
- Set `CONTAINER_HOST=unix:///run/user/<host-uid>/podman/podman.sock`.

In that setup, `scripts/mempalace.sh` can use the host container runtime without full nested container support.

## Git Push Ergonomics

- Ensure SSH auth is available.
- Ensure GitHub host trust is available.
- A temporary `known_hosts` file is a workaround.
- Better sandbox setup: writable `~/.ssh/known_hosts` or pre-seeded GitHub host keys.

## Host Services

- Use `host.containers.internal` for host-local TCP services.
- Do not assume `localhost` means the host. In the sandbox it is container-local.
