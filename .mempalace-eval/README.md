# MemPalace Evaluation Storage

This directory is the repo-local home for MemPalace evaluation data.

Purpose:

- keep generated MemPalace state outside ephemeral containers
- make retry runs use the same local palace and cache directories
- keep evaluation artifacts near the Riotbox repo instead of hidden in a user-global path

Current layout:

- `palace/`
  persistent Chroma database and indexed drawers
- `cache/`
  downloaded model and Python-side cache data
- `results/`
  captured command outputs from evaluation runs
- `corpus/`
  copied Riotbox source material used for mining

Important:

- this directory is intentionally inside the repo so the data persists on disk
- generated data stays ignored by Git because it is large and machine-specific
- the canonical project sources remain `docs/`, `plan/`, Linear, and Git history

Recommended container mounts for repeatable local trials:

- repo root -> `/repo`
- `.mempalace-eval/palace` -> `/palace`
- `.mempalace-eval/cache` -> `/root/.cache`

The current evaluation used a rootless Podman container with pinned `python:3.12-slim`.
