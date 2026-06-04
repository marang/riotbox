# `RIOTBOX-1187` Prototype CLAP/MERT-style Riotbox audio judge spike

- Ticket: `RIOTBOX-1187`
- Title: `Prototype CLAP/MERT-style Riotbox audio judge spike`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1187/prototype-clapmert-style-riotbox-audio-judge-spike`
- Project: `P021 | Audio Judge / Musical Fitness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1187-prototype-clapmert-style-riotbox-audio-judge-spike`
- Linear branch: `feature/riotbox-1187-prototype-clapmert-style-riotbox-audio-judge-spike`
- Assignee: `Markus`
- Labels: None
- PR: `#1165 (https://github.com/marang/riotbox/pull/1165)`
- Merge commit: `29790d70`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/prototype_audio_judge_spike.py; just audio-judge-spike-fixtures; just audio-judge-spike-generated-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/audio_judge_spike_v1_2026-06-04.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md; docs/README.md`
- Follow-ups: `RIOTBOX-1188 should define musical-pass gate policy; future P021 work needs real weak/fail labeled generated packs before provider calibration can claim usefulness.`

## Why This Ticket Existed

Prototype whether CLAP/MERT-style audio-judge work should augment Riotbox metrics without becoming runtime product truth.

## What Shipped

- Added riotbox.audio_judge_spike.v1 offline report, deterministic metrics baseline, optional provider availability reporting, label coverage/confusion examples, committed fixtures, generated dense-break smoke, and audio-qa-ci wiring.

## Notes

- Current recommendation is not_ready by design: only fixture labels exist, generated pack matches one pass label, weak/fail generated packs are still missing, and optional embedding providers are not CI dependencies.
