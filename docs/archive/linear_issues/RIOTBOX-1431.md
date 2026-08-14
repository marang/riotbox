# `RIOTBOX-1431` Workflow: Modularize agent-facing docs and reduce mandatory context cost

- Ticket: `RIOTBOX-1431`
- Title: `Workflow: Modularize agent-facing docs and reduce mandatory context cost`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1431/workflow-modularize-agent-facing-docs-and-reduce-mandatory-context`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-11`
- Started: `2026-08-11`
- Finished: `2026-08-11`
- Branch: `feature/riotbox-1431-workflow-modularize-agent-facing-docs-and-reduce-mandatory`
- Linear branch: `feature/riotbox-1431-workflow-modularize-agent-facing-docs-and-reduce-mandatory`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`, `Docs`, `Improvement`
- PR: `#1386 (https://github.com/marang/riotbox/pull/1386)`
- Merge commit: `be7f937fe9ea254c3133f6787ee8936cbc4dd7db`
- Deleted from Linear: `2026-08-14`
- Verification: `GitHub Rust CI passed cargo fmt, full cargo test, the audio-QA smoke gate, and strict Clippy. Skill validation, Decision-search fixtures, link checks, size assertions, benchmark immutability, and three independent scoped reviews passed.`
- Docs touched: `AGENTS.md; .codex/skills/; docs/workflow/; docs/specs/audio_qa/; docs/engineering/percussive_force/; docs/reviews/riotbox_1431_agent_context_modularization_2026-08-11.md`
- Follow-ups: `None; no additional ticket was started by this closeout.`

## Why This Ticket Existed

Riotbox agents repeatedly loaded overlapping workflow, audio-safety, and
production guidance. That consumed context, increased drift risk, and helped
turn bounded product work into oversized support machinery. The ticket reduced
that mandatory load while preserving every safety and product invariant.

## What Shipped

- Reduced the mandatory `AGENTS.md` plus development and production skill
  bundle from 48,762 bytes / 6,777 words to 14,098 / 1,810, a reduction of
  71.09% / 73.29% against the actual branch baseline.
- Split workflow, audio-QA, and percussive-force material into compact routing
  entry points with authoritative task-specific modules loaded only when
  relevant.
- Hardened bounded Decision Log lookup for exact suffixed IDs, section
  delimiters, shell metacharacters, and glob expansion, with regression
  fixtures.
- Made cross-ticket continuation explicitly opt-in so finishing one bounded
  request cannot silently grow into another implementation ticket.

## Bounded Outcome

- No audio, source corpus, runtime, product path, schema, threshold, frozen
  algorithm, or benchmark JSON changed.
- No human listening was needed because the branch was maintenance/regression
  work with no audible consequence.
- Three independent scoped reviews reported no remaining findings, and GitHub
  CI passed before merge.

## Links

- [Agent-context modularization audit](../../reviews/riotbox_1431_agent_context_modularization_2026-08-11.md)
- [Workflow conventions](../../workflow_conventions.md)
- [Audio QA workflow](../../specs/audio_qa_workflow_spec.md)
