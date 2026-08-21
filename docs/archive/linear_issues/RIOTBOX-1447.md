# `RIOTBOX-1447` P023: Qualify the kept W-30 gesture vocabulary in the live Golden Path

- Ticket: `RIOTBOX-1447`
- Title: `P023: Qualify the kept W-30 gesture vocabulary in the live Golden Path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1447/p023-qualify-the-kept-w-30-gesture-vocabulary-in-the-live-golden-path`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-21`
- Started: `2026-08-21`
- Finished: `2026-08-21`
- Branch: `feature/riotbox-1447-p023-qualify-the-kept-w-30-gesture-vocabulary-in-the-live`
- Linear branch: `feature/riotbox-1447-p023-qualify-the-kept-w-30-gesture-vocabulary-in-the-live`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1414 (https://github.com/marang/riotbox/pull/1414)`
- Merge commit: `01691c71bc439568bf5817330dc3e2793b97d719`
- Deleted from Linear: `2026-08-21`
- Verification: `just ci: passed; focused Core/audio tests and workspace Clippy: passed; GitHub Rust CI: passed; frozen Development-only gesture-vocabulary qualification: passed; focused human re-entry confirmation: passed with intentional fresh-trigger onset documented`
- Docs touched: `docs/benchmarks/w30_gesture_vocabulary_golden_path_qualification_v1.json, docs/reviews/riotbox_1447_w30_gesture_reentry_qualification_2026-08-21.md, docs/specs/action_lexicon_spec.md, docs/specs/replay_model_spec.md, README.md`
- Follow-ups: `Gesture-combination taste remains unverified and should be evaluated only for a concrete future musical use case.`

## Why This Ticket Existed

The three individually kept W-30 gestures needed exact live-path transition proof; source-blind inspection found that Pitch Dive terminal silence could persist after an ordinary W-30 action.

## What Shipped

- Ordinary W-30 recall, trigger, audition, and damage actions now supersede timed articulations identically in live state and Session/replay.
- Added a frozen Development-only three-gesture Golden Path qualification with exact access, callback, limiter, replay, and RuntimeMix proof.
- Recorded a focused human confirmation that normal playback returns after Pitch Dive, while preserving the stronger fresh-trigger onset and deferring combination taste.

## Notes

- No gesture DSP, duration, mapping, threshold, or automatic ordering changed. No Holdout or commercial-reference audio was accessed.
