# `RIOTBOX-1277` MC-202 quality gate rejects source-fake and hardcoded composer output

- Ticket: `RIOTBOX-1277`
- Title: `MC-202 quality gate rejects source-fake and hardcoded composer output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1277/mc-202-quality-gate-rejects-source-fake-and-hardcoded-composer-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1277-mc202-source-fake-quality-gate`
- Linear branch: `feature/riotbox-1277-mc-202-quality-gate-rejects-source-fake-and-hardcoded`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1252 (https://github.com/marang/riotbox/pull/1252)`
- Merge commit: `1ac9ae95ee4e728848d21f1212f875125380f2bc`
- Deleted from Linear: `2026-06-15`
- Verification: `git diff --check; cargo test -p riotbox-audio 'w30|mc202|source_monitor' -- --nocapture; cargo test -p riotbox-app 'w30|mc202|source_monitor' -- --nocapture; cargo test -p riotbox-audio feral_grid_pack -- --nocapture; cargo clippy --all-targets --all-features -- -D warnings; just professional-output-listening-pack-smoke; just professional-output-suite-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `AGENTS.md; docs/plans/mc202_source_phrase_planning_plan.md; docs/specs/action_lexicon_spec.md; docs/specs/audio_core_spec.md; docs/specs/tui_screen_spec.md; .codex/skills/riotbox-development/SKILL.md; .codex/skills/riotbox-rave-punk-production/SKILL.md`
- Follow-ups: `RIOTBOX-1280, RIOTBOX-1281, RIOTBOX-1282, RIOTBOX-1283, RIOTBOX-1284`

## Why This Ticket Existed

Remove hardcoded musical/audio fallback output from Riotbox product paths so unavailable source intelligence is visible instead of masked by fake replacement sound.

## What Shipped

- MC-202 renders only source phrase plans; W-30 preview requires source/pad samples; source monitor reports source_unavailable and mutes unavailable source/blend routes; professional-output suite blocks template-only tonal hook cases from promotion.

## Notes

- None
