# MemPalace Evaluation Storage

This directory is the repo-local home for MemPalace data.

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
- `.mempalace/palace` -> `/palace`
- `.mempalace/cache` -> `/root/.cache`

The current evaluation uses a rootless Podman setup with pinned
`python:3.14.4-slim` and `mempalace==3.3.4`.

Repo tooling now exists for this:

- `compose.mempalace.yaml`
- `Containerfile.mempalace`
- `scripts/mempalace.sh`
- `just mem-init`
- `just mem-repair`
- `just mem-status`
- `just mem-search "query"`

The wrapper script automatically:

- syncs selected live repo sources into room-specific folders under `.mempalace/corpus/`
- excludes `docs/archive/linear_issues/` from the mined corpus so ticket-history cleanup does not dominate retrieval
- writes the Riotbox room structure into the generated corpus before mining
- re-mines the corpus if those sources changed since the last successful index
- serializes operations with a repo-local lock so multiple status/search calls do not mine concurrently
- rebuilds the palace index when the room structure changes
- repairs the palace index through `just mem-repair` when MemPalace reports
  index metadata drift, such as missing cosine-distance metadata
- rebuilds the MemPalace container image only when `Containerfile.mempalace` or `compose.mempalace.yaml` changes
- runs normal runtime commands with container networking disabled; image builds
  still require normal registry/network access

Current rooms:

- `specs`
  Product, architecture, runtime, audio, and workflow specs under `docs/specs/`
- `workflow`
  Agent, GitHub, Linear, MemPalace, and repository operating conventions
- `reviews`
  Codebase, seam, MVP exit, and periodic review artifacts
- `audio_qa`
  Audio QA, listening packs, benchmarks, probes, manifests, and output-proof material
- `plan`
  Strategy, roadmap, masterplan, and phase planning material
- `decisions`
  Decision logs, spikes, research notes, and frozen technical choices
- `code`
  Rust crate source and test implementation details
- `documentation`
  Product-facing docs, recipes, README material, and general documentation
- `general`
  Files that do not fit a more specific Riotbox room
