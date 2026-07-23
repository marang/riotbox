# `RIOTBOX-1408` P023: Replace the synthetic W-30 resample tap voice with source-backed resample audio

- Ticket: `RIOTBOX-1408`
- Title: `P023: Replace the synthetic W-30 resample tap voice with source-backed resample audio`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1408/p023-replace-the-synthetic-w-30-resample-tap-voice-with-source-backed`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-07-21`
- Finished: `2026-07-23`
- Branch: `feature/riotbox-1408-p023-replace-the-synthetic-w-30-resample-tap-voice-with`
- Linear branch: `feature/riotbox-1408-p023-replace-the-synthetic-w-30-resample-tap-voice-with`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1375 (https://github.com/marang/riotbox/pull/1375)`
- Merge commit: `486983660395d2ff9b60db59aa99906d8df05282`
- Deleted from Linear: `2026-07-23`
- Verification: `just ci; just audio-qa-ci; focused W-30 resample tests; cargo clippy --all-targets --all-features -- -D warnings; GitHub rust-ci; exact three-source/determinism/missing-source proof; structured human weak verdict`
- Docs touched: `docs/reviews/riotbox_1408_source_backed_w30_resample_review_2026-07-21.md; docs/specs/audio_core_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1422`

## Why This Ticket Existed

The existing W-30 internal resample tap was a fixed-frequency oscillator/shimmer scaffold presented on a playable product path instead of audio from the committed capture lineage.

## What Shipped

- Replaced the synthetic tap voice with a deterministic transient-selected 4096-sample grain hydrated from the committed W-30 capture artifact and rendered through the exact callback/RuntimeMix seam.
- Preserved promote.resample queue/commit, Session/replay, capture lineage, observer/TUI state, and exact transport timing while routing missing or invalid source audio to typed unavailable state and digital silence.
- Proved byte-stable replay, non-silent unclipped output across Beat03/Beat08/Beat20, cross-source diversity, raw-source non-collapse, retired-proxy non-collapse, and no synthetic fallback.

## Notes

- Structured direct source-to-tap listening recorded technically_ok_but_musically_weak: the chop was interesting but very timid, source recognition was lost, and the hook was weak. Demo-ready promotion remains blocked; RIOTBOX-1422 owns the audible correction.
