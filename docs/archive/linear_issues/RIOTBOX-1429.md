# `RIOTBOX-1429` P023 Research: Define percussive force, beat impact, and analysis

- Ticket: `RIOTBOX-1429`
- Title: `P023 Research: Define percussive force, beat impact, and analysis`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1429/p023-research-define-percussive-force-beat-impact-and-analysis`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-02`
- Started: `2026-08-02`
- Finished: `2026-08-03`
- Branch: `feature/riotbox-1429-p023-research-define-and-validate-source-adaptive-percussive`
- Linear branch: `feature/riotbox-1429-p023-research-define-percussive-force-beat-impact-and`
- Assignee: `Owner`
- Labels: `Analysis`, `Audio`, `Docs`, `Spike`, `review-followup`
- PR: `#1383 (https://github.com/marang/riotbox/pull/1383)`
- Merge commit: `f79cc77e6b43280543052bcb701ca3d9d6a145e3`
- Deleted from Linear: `2026-08-14`
- Verification: `Final branch review, GitHub Rust CI including the audio-QA smoke gate, cargo fmt, cargo test, strict Clippy, source-holdout rotation fixtures, JSON duplicate-key checks, link checks, and decision/evidence ID checks passed. The active holdout manifest hash remained unchanged.`
- Docs touched: `docs/engineering/percussive_force_and_beat_impact.md; docs/benchmarks/percussive_force_development_matrix_v1.json; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md; docs/execution_roadmap.md; docs/phase_definition_of_done.md; AGENTS.md; .codex/skills/riotbox-rave-punk-production/SKILL.md`
- Follow-ups: `RIOTBOX-1428`

## Why This Ticket Existed

Repeated H27--H31 Hard experiments produced measurable differences that human
review did not hear as a more forceful strike. Riotbox needed a causal,
evidence-labeled account of percussive force and whole-beat impact before any
further DSP implementation or scalar tuning.

## What Shipped

- A canonical research paper separating percussive force, timbral hardness,
  punch, body, bass pressure, hook hardness, aggression, groove, heaviness, and
  arrangement impact.
- An unexecuted, multi-source F1--F3 Stage-A preregistration draft with legal
  controls, falsifiers, stopping rules, and an untouched holdout boundary.
- Durable QA guardrails across the project skill, agent brief, audio QA spec,
  roadmap, phase definition of done, and decision log.
- A strict handoff to RIOTBOX-1428: only a recognizable event heard as more
  forcefully struck with body, bite, identity, and timing retained can satisfy
  `percussive_hard`; `hook_hard` cannot substitute for that verdict.

## Bounded Outcome

- No Rust, DSP, renderer, realtime, TUI, Session, replay, or product behavior changed.
- No candidate audio was rendered and no human playback was requested.
- No algorithm family was selected and no musical-quality or instrument-progress claim was made.
- H27--H31 remain bounded, hash-linked design observations rather than executable historical controls.

## Links

- [Canonical paper](../../engineering/percussive_force_and_beat_impact.md)
- [Stage-A preregistration draft](../../benchmarks/percussive_force_development_matrix_v1.json)
- [H31 rejected-experiment record](../../reviews/riotbox_1428_h31_stage_a_rejected_experiment_2026-08-02.md)
