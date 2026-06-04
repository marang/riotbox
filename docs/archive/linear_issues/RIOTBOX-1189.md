# `RIOTBOX-1189` Add weak/fail audio judge calibration packs

- Ticket: `RIOTBOX-1189`
- Title: `Add weak/fail audio judge calibration packs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1189/add-weakfail-audio-judge-calibration-packs`
- Project: `P021 | Audio Judge / Musical Fitness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1189-add-weakfail-audio-judge-calibration-packs`
- Linear branch: `feature/riotbox-1189-add-weakfail-audio-judge-calibration-packs`
- Assignee: `Markus`
- Labels: None
- PR: `#1167 (https://github.com/marang/riotbox/pull/1167)`
- Merge commit: `1ff9d5de`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/prototype_audio_judge_spike.py scripts/validate_human_listening_label_corpus.py; just human-listening-label-corpus-fixtures; just audio-judge-spike-fixtures; just audio-judge-spike-generated-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/audio_judge_spike_v1_2026-06-04.md; docs/benchmarks/human_listening_label_corpus_v1_2026-06-04.md`
- Follow-ups: `RIOTBOX-1190 should add cross-source label coverage; RIOTBOX-1191 should add real listening-label import so fixture labels do not become taste truth.`

## Why This Ticket Existed

Give the P021 audio judge matched pass, weak, and fail calibration examples instead of one generated pass match plus unmatched weak label.

## What Shipped

- Extended audio judge spike to accept multiple agent-review inputs with unique review_pack_id values; added dense-break weak/fail agent-review fixtures and matching human label corpus entries; tightened fixture and generated smoke assertions.

## Notes

- Labels remain fixture calibration evidence, not real human listening approval. Generated dense-break smoke still matches one pass label and reports weak/fail coverage gaps, so readiness remains not_ready.
