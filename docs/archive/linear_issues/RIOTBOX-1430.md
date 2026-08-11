# `RIOTBOX-1430` P023: Freeze Stage-A Protocol v2 and legal retry snapshot

- Ticket: `RIOTBOX-1430`
- Title: `P023: Freeze Stage-A Protocol v2 and legal retry snapshot`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1430/p023-freeze-stage-a-protocol-v2-and-legal-retry-snapshot`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-10`
- Started: `2026-08-10`
- Finished: `2026-08-11`
- Branch: `feature/riotbox-1430-stage-a-protocol-v2-retry-snapshot`
- Linear branch: `feature/riotbox-1430-p023-freeze-stage-a-protocol-v2-and-legal-retry-snapshot`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `Spike`, `review-followup`
- PR: `#1385 (https://github.com/marang/riotbox/pull/1385)`
- Merge commit: `d00fc35cde6482c7ee5218b1e6a2462862e2f107`
- Deleted from Linear: `Pending fresh token-authenticated cleanup; retained as Done`
- Verification: `GitHub Rust CI passed cargo fmt, full cargo test, the audio-QA smoke gate, and strict Clippy. Targeted Protocol-v2, Registry-v3, Matrix-v3, analysis, and qualification fixtures passed. Two independent branch reviews found no remaining blocking or major issues.`
- Docs touched: `docs/research_decision_log.md; docs/reviews/riotbox_1430_stage_a_v2_development_qualification_rejection_2026-08-11.md; AGENTS.md; .codex/skills/riotbox-development/SKILL.md`
- Follow-ups: `RIOTBOX-1431; any later percussive-force attempt requires a new Protocol-v3 decision`

## Why This Ticket Existed

The first RIOTBOX-1428 Stage-A admission failed under Protocol v1, and its
post-execution audit identified evidence-label, protocol-object, and source-set
problems that could only be corrected before a new source access. This ticket
froze one legal Protocol-v2 retry without retuning from source results.

## What Shipped

- Frozen Protocol-v2, Registry-v3, and Matrix-v3 contracts for four Development
  sources from four authors across three source families, while preserving the
  unopened active holdout union.
- One bounded Development-only qualification session. Cinameng and Djericmark
  qualified with three events each; Cyclez yielded one eligible event and
  Justabeat yielded none, so the four-source admission rejected fail-closed.
- A terminal rejection record that closes Matrix-v3 rendering and human
  playback without imputing events, substituting sources, or tuning the frozen
  algorithms.
- Removal of 15,791 added executable lines of consumed acquisition machinery
  and every runnable acquisition command. The durable project rule now keeps
  future one-off source acquisition operational and small.
- A typed post-RBX-263 runner refusal plus a source-blind regression fixture
  proving that no validation or access callback can be reached by a retry.

## Bounded Outcome

- No Matrix-v3 condition or candidate audio was rendered.
- No human hardness or musical-quality verdict exists.
- No RuntimeMix, realtime, TUI, Session, replay, or product-path behavior changed.
- No holdout audio, commercial reference, or source-directory discovery was used.
- A later attempt must version Protocol v3 and every affected component before
  any new source access; Protocol v2 cannot be retuned from this rejection.

## Links

- [Qualification rejection](../../reviews/riotbox_1430_stage_a_v2_development_qualification_rejection_2026-08-11.md)
- [Protocol v2](../../benchmarks/percussive_force_stage_a_protocol_v2.json)
- [Registry v3](../../benchmarks/source_holdout_rotation_v3.json)
- [Matrix v3](../../benchmarks/percussive_force_development_matrix_v3.json)
