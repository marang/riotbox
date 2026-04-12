# MemPalace Evaluation

Date: 2026-04-12  
Ticket: `RIOTBOX-16`  
Scope: evaluate MemPalace as an internal project-memory and agent-assist tool for Riotbox. This is not a product-core evaluation.

## Question

Should Riotbox adopt MemPalace soon as a local memory/search layer for project history, decisions, and agent-assist retrieval?

## Short Answer

Not yet for default workflow adoption.

Recommendation:

- do not adopt it as a standing Riotbox dependency right now
- keep it as a later optional experiment once we can test it on a supported Python runtime
- treat it as a possible external dev tool, never as a replacement for repo docs, Linear, or Riotbox core state

## Why

MemPalace is directionally aligned with the kind of project-memory problem we have:

- local-first memory
- searchable project and conversation history
- MCP-facing workflow
- explicit support for agent-assist usage

But for Riotbox right now, it has three practical problems:

1. It is still operationally noisy.
2. It introduces another system to maintain beside repo docs and Linear.
3. Our first real local trial did not reach successful indexing on this machine baseline.

That makes it interesting, but not yet worth standardizing.

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

## Real Trial

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

## Rootless Podman Follow-up

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
- mounted a small writable Riotbox corpus
- mounted a writable palace directory
- installed `mempalace==3.1.0` inside the container
- ran `mempalace init --yes`
- started `mempalace mine`

### What worked in the container

- the exact host-side `Python 3.14` compatibility failure did not occur
- `mempalace init --yes` completed successfully
- `mempalace mine` started successfully inside the container
- Chroma began downloading its embedding model during the mine step, which means the runtime got materially further than the host trial

### What was not completed in this follow-up

At the time of writing, the rootless container follow-up had not yet completed the full download-and-index cycle through to a finished search result. The remaining wait was model download time, not the earlier runtime compatibility crash.

### Updated conclusion

Rootless Podman looks like a viable mitigation for the host runtime problem.

That changes the evaluation slightly:

- MemPalace is still not recommended as a default Riotbox workflow dependency right now
- but a containerized retry path is clearly more promising than the raw host install path

So the refined recommendation is:

- park it for now as a standard workflow tool
- if we revisit it, use a pinned containerized environment first

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

A lot of MemPalace’s workflow framing is Claude Code-centric. Riotbox work here is happening in Codex plus Linear plus GitHub. That does not make MemPalace unusable, but it reduces immediate plug-and-play value.

## Recommendation

Status: park for now, revisit later.

Concrete recommendation:

- do not adopt MemPalace as part of the normal Riotbox workflow yet
- do not add it to the repo tooling baseline
- do not route any product state through it
- keep the ticket outcome as: interesting external dev-memory candidate, but not ready for standard use on our current machine/runtime baseline

## Revisit Triggers

Re-evaluate MemPalace later if at least one of these becomes true:

- we have a supported Python runtime available locally for a clean second trial
- upstream explicitly supports newer Python versions we actually use
- we feel repeated pain from missing long-term project memory that repo docs + Linear + issue comments are not solving
- we want a dedicated optional agent-assist retrieval helper and are willing to isolate it in its own environment

## If We Test Again

The next trial should be stricter and smaller:

1. use a supported Python runtime, ideally `3.11` or `3.12`
2. keep a tiny Riotbox corpus
3. validate three exact tasks:
   - retrieve a prior architecture decision
   - retrieve a prior roadmap or phase decision
   - retrieve a past implementation rationale across files
4. compare that result against plain `rg` plus repo docs plus Linear history

Only if it clearly beats the current workflow should we consider operationalizing it.
