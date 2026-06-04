# `RIOTBOX-1190` Add cross-source audio judge label coverage

- Ticket: `RIOTBOX-1190`
- Title: `Add cross-source audio judge label coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1190/add-cross-source-audio-judge-label-coverage`
- Project: `P021 | Audio Judge / Musical Fitness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1190-add-cross-source-audio-judge-label-coverage`
- Linear branch: `feature/riotbox-1190-add-cross-source-audio-judge-label-coverage`
- Assignee: `Markus`
- Labels: None
- PR: `#1168 (https://github.com/marang/riotbox/pull/1168)`
- Merge commit: `5d695569`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/prototype_audio_judge_spike.py scripts/validate_human_listening_label_corpus.py; just human-listening-label-corpus-fixtures; just audio-judge-spike-fixtures; just audio-judge-spike-generated-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/audio_judge_spike_v1_2026-06-04.md; docs/benchmarks/human_listening_label_corpus_v1_2026-06-04.md`
- Follow-ups: `RIOTBOX-1191 should add a real listening-label import path so fixture coverage can be replaced by structured human labels.`

## Why This Ticket Existed

Prevent the audio judge from learning only dense_break fixture shape by adding cross-source calibration coverage.

## What Shipped

- Added source-family coverage to audio judge reports, added sparse_bass_pressure as a label source family, and added tonal_hook pass plus sparse_bass_pressure weak fixtures and labels.

## Notes

- Cross-source coverage is still fixture-only calibration evidence. Generated dense-break smoke continues to expose dense_break-only matched coverage and missing cross-source examples.
