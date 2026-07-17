# `RIOTBOX-1401` P023: Add a curated Feral Break Alpha preset with capture, recall, and replay

- Ticket: `RIOTBOX-1401`
- Title: `P023: Add a curated Feral Break Alpha preset with capture, recall, and replay`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1401/p023-add-a-curated-feral-break-alpha-preset-with-capture-recall-and`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-17`
- Finished: `2026-07-17`
- Branch: `feature/riotbox-1401-p023-feral-break-alpha-preset`
- Linear branch: `feature/riotbox-1401-p023-add-a-curated-feral-break-alpha-preset-with-capture`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `TUI`, `ux`
- PR: `#1368 (https://github.com/marang/riotbox/pull/1368)`
- Merge commit: `e11c02290f4ba492ad684c3adac3f0b7bba3db0b`
- Deleted from Linear: `2026-07-17`
- Verification: `just ci; just audio-qa-ci; just check; generated and real exact-path validators; GitHub Rust CI`
- Docs touched: `README.md; docs/jam_recipes.md; docs/research_decision_log.md; action, preset, replay, session, and source-timing specs`
- Follow-ups: `RIOTBOX-1402 owns structured exact live-path listening and the human musical-pass gate`

## Why This Ticket Existed

Provide one typed, recallable P023 performance state on the existing product spine and prove its exact live mixer, capture, restart, and replay journey.

## What Shipped

- Typed Feral Break Alpha profile/preset through ActionCommand, queue/commit, Session/replay, TUI, and observer.
- Exact eight-bar RuntimeMix proof with hook, drum-pressure lift, destructive contrast, changed return, raw/RMS-matched A/B, and explicit bass ownership.
- Capture, raw audition, promotion, save, restart, live recall, and trigger proof with deterministic hashes and no clipping or limiter activity.
- Scene-owned short-source MC-202 phrase selection that fails closed without trusted section/grid overlap.

## Notes

- human_verdict remains unverified; no musical-pass or recognizable Riotbox-character claim was made.
