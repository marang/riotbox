# `RIOTBOX-1188` Define musical-pass gate policy for agent and human verdicts

- Ticket: `RIOTBOX-1188`
- Title: `Define musical-pass gate policy for agent and human verdicts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1188/define-musical-pass-gate-policy-for-agent-and-human-verdicts`
- Project: `P021 | Audio Judge / Musical Fitness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1188-define-musical-pass-gate-policy-for-agent-and-human-verdicts`
- Linear branch: `feature/riotbox-1188-define-musical-pass-gate-policy-for-agent-and-human-verdicts`
- Assignee: `Markus`
- Labels: None
- PR: `#1166 (https://github.com/marang/riotbox/pull/1166)`
- Merge commit: `f4269f86`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/validate_musical_pass_gate_policy.py; just musical-pass-gate-policy-fixtures; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/musical_pass_gate_policy_v1_2026-06-04.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md; docs/README.md`
- Follow-ups: `Future P021 judge work needs real labeled pass/weak/fail generated packs and source-family-specific validation before calibrated_agent_musical_pass can be claimed.`

## Why This Ticket Existed

Define exact verdict language so agents can block weak audio without overclaiming musical_pass from logs, simple metrics, or uncalibrated embeddings.

## What Shipped

- Added riotbox.musical_pass_gate_policy.v1, a validator, positive/negative fixtures, just musical-pass-gate-policy-fixtures wired into audio-qa-ci, and audio-QA/benchmark/decision-log documentation.

## Notes

- Only human_musical_pass and bounded calibrated_agent_musical_pass may claim musical pass. agent_promising remains useful merge evidence but keeps human_verdict unverified.
