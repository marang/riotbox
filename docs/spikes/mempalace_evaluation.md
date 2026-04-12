# MemPalace Evaluation

Date: 2026-04-12  
Ticket: `RIOTBOX-16`  
Scope: evaluate MemPalace as an internal project-memory and agent-assist tool for Riotbox. This is not a product-core evaluation.

## Question

Should Riotbox adopt MemPalace soon as a local memory/search layer for project history, decisions, and agent-assist retrieval?

## Short Answer

Yes for real evaluation value, but not yet as a mandatory default workflow dependency.

Recommendation:

- keep it explicitly out of Riotbox product core
- keep repo docs, Linear, and Git history as the canonical sources of truth
- treat MemPalace as a promising optional dev-memory layer, with the currently recommended setup being rootless Podman plus pinned `python:3.12`
- revisit whether it should become part of the normal team workflow after a broader retrieval bakeoff, not after one host-only failure

## Why

MemPalace is directionally aligned with the kind of project-memory problem we have:

- local-first memory
- searchable project and conversation history
- MCP-facing workflow
- explicit support for agent-assist usage

But for Riotbox right now, it still has three practical constraints:

1. The raw host install path is not a safe default on this machine baseline.
2. It introduces another system to maintain beside repo docs and Linear.
3. We have one successful containerized trial so far, not a broader proof that it beats the current workflow across daily work.

That makes it promising, but not yet something I would force into the default workflow for every contributor.

## What Was Reviewed

Primary sources:

- MemPalace README
- MemPalace `pyproject.toml`
- MemPalace roadmap
- issue tracker around stability and security

Key observed themes from upstream:

- the project is explicitly local-first and MCP-oriented
- the maintainers already corrected earlier overstated claims in the README
- `chromadb` remains the default backend in `3.1.0`
- there is active work toward backend abstraction and storage swaps
- there are still open or recently active issues around hook hardening, macOS ARM stability, HNSW index behavior, and platform compatibility

## Riotbox Fit

### Where it could help

- searching old planning and architecture discussions
- surfacing prior decisions across long agent sessions
- reducing repeated context rebuilding for non-product-state knowledge
- giving agents a local searchable memory without pushing more logic into Riotbox itself

### Where it does not help

- `SourceGraph`
- `SessionFile`
- replay model
- action log
- audio/runtime state
- Ghost execution semantics

Those remain Riotbox-native contracts and should not move into a third-party memory tool.

## Host Trial

### Trial goal

Try a small local Riotbox corpus and verify whether MemPalace can be installed, initialized, mined, and queried with minimal setup.

### Corpus used

Small Riotbox doc subset copied to `/tmp/riotbox-mempalace-corpus`:

- `docs/prd_v1.md`
- `docs/execution_roadmap.md`
- `docs/research_decision_log.md`
- `docs/specs/technology_stack_spec.md`

### What worked

- created an isolated virtualenv
- installed `mempalace==3.1.0`
- inspected CLI commands successfully
- ran `mempalace init --yes` far enough to see entity detection output

### What failed

The trial did not reach a successful mine/search cycle on this machine.

Observed failure:

- machine default Python is `3.14.4`
- `mempalace 3.1.0` installs, but `chromadb` fails during runtime import with:
  `pydantic.v1.errors.ConfigError: unable to infer type for attribute "chroma_server_nofile"`

Observed warning before failure:

- `Core Pydantic V1 functionality isn't compatible with Python 3.14 or greater.`

Practical result:

- install succeeded
- initialization partially ran
- mining and therefore real retrieval validation did not complete on this host baseline

### What this means

For Riotbox, MemPalace is not currently a low-friction tool we can assume will work on any developer machine without extra runtime curation.

That matters because a dev-memory helper only earns its keep if setup is boring.

## Rootless Podman Evaluation

### Follow-up question

Can rootless Podman provide a workable environment even if the host Python baseline is too new?

### What was tested

Rootless Podman was checked directly on this machine.

Observed host facts:

- `podman` is installed
- Podman reports `rootless: true`
- container runtime, storage, and rootless networking are configured normally on this host

Follow-up container trial:

- image: `python:3.12-slim`
- runtime: rootless Podman
- mounted repo-local persistent directories under `.mempalace-eval/`
- installed `mempalace==3.1.0` inside the container
- ran `mempalace init --yes`
- completed `mempalace mine`
- ran real search queries against Riotbox data

### Repo-local persistent storage

The container was configured so data persisted in the repo, not only inside the container filesystem:

- `.mempalace-eval/palace/` stores the persistent Chroma database
- `.mempalace-eval/cache/` stores downloaded model/cache data
- `.mempalace-eval/results/` stores captured run outputs
- `.mempalace-eval/corpus/` stores the copied Riotbox evaluation corpus

This keeps the evaluation repeatable without depending on container-local state.

### What worked in the container

- the exact host-side `Python 3.14` compatibility failure did not occur
- `mempalace init --yes` completed successfully
- `mempalace mine` completed successfully
- `mempalace status` reported a populated palace
- real search queries returned relevant Riotbox documents

Observed result:

- `487` indexed drawers
- room split:
  - `plan`: `279`
  - `documentation`: `207`
  - `general`: `1`

Result files:

- `.mempalace-eval/results/init.txt`
- `.mempalace-eval/results/mine.txt`
- `.mempalace-eval/results/status.txt`
- `.mempalace-eval/results/search_rust.txt`
- `.mempalace-eval/results/search_feral.txt`
- `.mempalace-eval/results/search_source_graph.txt`

### Query quality snapshot

Three real queries were tested against Riotbox data:

1. `Why Rust for the main core`
2. `feral_rebuild profile`
3. `source graph session action`

Observed quality:

- query 1 returned strong hits immediately, including `rust_engineering_guidelines.md`, `research_decision_log.md`, and `technology_stack_spec.md`
- query 2 returned useful hits around the feral profile, especially `preset_style_spec.md` and the active/planned feral addenda
- query 3 returned relevant but somewhat noisier hits, with `session_file_spec.md` and `source_graph_spec.md` near the top

This is good enough to prove real utility. It is not yet enough to prove that MemPalace should become a mandatory default workflow layer.

### Updated conclusion

Rootless Podman is not just a mitigation idea. It is a viable working setup for Riotbox on this machine.

That changes the evaluation materially:

- MemPalace is now proven usable for Riotbox dev-memory in a containerized setup
- the earlier host failure should be treated as a host-runtime problem, not a tool-wide rejection
- the remaining question is no longer basic operability, but workflow value versus maintenance cost

So the refined recommendation is:

- do not make MemPalace a required default dependency for every contributor yet
- do treat it as a credible optional dev-memory tool now
- if we operationalize it, use a pinned rootless-container workflow first

## Operational Risks

### 1. Runtime compatibility risk

The first trial already failed on Python `3.14`.

Even if the project works well on `3.9` to `3.12`, Riotbox would need to carry an explicit environment constraint or containerized wrapper for it. That is non-trivial operational weight for a non-core helper.

### 2. Storage/backend churn

Upstream is actively evolving storage seams and discussing multiple backend directions. That is healthy, but it also means the tool is still settling.

For Riotbox, that means we should avoid binding internal workflow too tightly to it yet.

### 3. It can become a second truth system

If used casually, MemPalace could duplicate:

- repo docs
- Linear history
- decision log material

That would create ambiguity instead of reducing it.

If Riotbox ever uses it, it should only index and retrieve from existing sources of truth. It should not become a hidden place where new canonical decisions live.

### 4. Hook and integration bias

A lot of MemPalace’s workflow framing is Claude Code-centric. Riotbox work here is happening in Codex plus Linear plus GitHub. That does not make MemPalace unusable, but it still means we should adopt it deliberately rather than assume upstream defaults fit our workflow unchanged.

## Recommendation

Status: validated as a working optional tool; do not require it by default yet.

Concrete recommendation:

- do not adopt MemPalace as a required baseline dependency for all contributors yet
- do not add it to the Riotbox product/runtime tooling baseline
- do not route any product state through it
- keep using repo docs, Linear, and Git history as canonical sources
- keep the currently recommended MemPalace setup as: rootless Podman, pinned `python:3.12`, repo-local persistent storage under `.mempalace-eval/`
- treat the ticket outcome as: usable optional dev-memory candidate, worth further evaluation against real day-to-day retrieval tasks

## Revisit Triggers

Broader adoption should be reconsidered if at least one of these becomes true:

- it proves clearly faster or better than `rg` plus repo docs plus Linear across repeated real tasks
- we add a small wrapper script so the container path becomes boring to run
- multiple developers want the same shared local retrieval helper
- upstream improves compatibility enough that the host-install path becomes low-friction again

## If We Test Again

The next trial should be broader and comparative:

1. keep using the pinned rootless-container setup
2. run at least ten real retrieval tasks from active Riotbox work
3. compare task speed and answer quality against plain `rg` plus repo docs plus Linear history
4. decide whether it becomes:
   - optional helper only
   - recommended dev tool
   - or still not worth the maintenance cost

Only if it clearly beats the current workflow should we consider operationalizing it.
