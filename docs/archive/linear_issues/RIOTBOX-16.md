# `RIOTBOX-16` Ticket Archive

- Ticket: `RIOTBOX-16`
- Title: `Evaluate MemPalace for project memory and agent-assist knowledge retrieval`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-16/evaluate-mempalace-for-project-memory-and-agent-assist-knowledge`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-12`
- Finished: `2026-04-12`
- Branch: `riotbox-16-mempalace-evaluation`
- Assignee: `Markus`
- Labels: `Infra`, `Spike`, `Docs`
- PR: `#10`
- Merge commit: `bc6f0b6`
- Verification: `isolated MemPalace install`, `rootless Podman trial`, `mine`, `status`, `search`, broader bakeoff against repo data`
- Docs touched: `.mempalace/README.md`, `docs/spikes/mempalace_evaluation.md`, `docs/research_decision_log.md`, `docs/README.md`, `AGENTS.md`, `Justfile`, `scripts/mempalace.sh`, `Containerfile.mempalace`, `compose.mempalace.yaml`
- Follow-ups: `None`

## Why This Ticket Existed

Riotbox needed a bounded evaluation of MemPalace as an internal project-memory and agent-assist retrieval tool without turning it into product-core infrastructure.

## What Shipped

- Evaluated MemPalace against real Riotbox repository data instead of only reading upstream documentation.
- Confirmed the host Python 3.14 path was unstable for the toolchain in this environment.
- Confirmed a rootless Podman path with pinned Python 3.12 worked and was the right operational baseline.
- Added a real bakeoff against Riotbox retrieval tasks and documented where MemPalace helps versus where `rg` stays better.
- Added repo-local wrapper and `just` integration so MemPalace became operationally usable as an optional dev-memory tool.

## Notes

- Outcome: adopt as an optional repo tool, not canonical truth and not a replacement for `rg`.
- This ticket changed workflow and retrieval practice, but not Riotbox product runtime or data models.
