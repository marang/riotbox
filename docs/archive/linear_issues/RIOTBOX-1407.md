# `RIOTBOX-1407` P023: Route live TUI transport through Action Lexicon commit and replay

- Ticket: `RIOTBOX-1407`
- Title: `P023: Route live TUI transport through Action Lexicon commit and replay`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1407/p023-route-live-tui-transport-through-action-lexicon-commit-and-replay`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-08-22`
- Finished: `2026-08-22`
- Branch: `feature/riotbox-1407-p023-route-live-tui-transport-through-action-lexicon-commit`
- Linear branch: `feature/riotbox-1407-p023-route-live-tui-transport-through-action-lexicon-commit`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `TUI`, `review-followup`
- PR: `#1430 (https://github.com/marang/riotbox/pull/1430)`
- Merge commit: `3f0733a2eea96d5d86a7848428a3fd537a92a476`
- Deleted from Linear: `2026-08-22`
- Verification: `Full just ci passed locally after the fixed observer contracts were updated.`; `GitHub Rust CI passed on PR #1430.`; `643 riotbox-app library tests, all 15 user-session observer probes, Stage Style probe chain, Clippy -D warnings, formatting, and code/Rust review passed with no remaining findings.`
- Docs touched: `docs/README.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/replay_model_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-1334 is the next and final named Foundation Completion gap.`

## Why This Ticket Existed

Repair the live Space Play/Pause path so musician transport input cannot bypass Action Lexicon commit, observer, and replay truth.

## What Shipped

- Added one shared immediate transport-toggle commit entry that enqueues existing TransportPlay or TransportPause actions at the current clock boundary.
- Aligned Session/runtime transport, audio-driver pending state, observer commits, and replay on the same action identity without a new transport action or boundary delay.
- Strengthened First Playable and Stage Style validators to require the new Immediate transport action and updated owning specs and Foundation ordering.

## Notes

- No Development/Holdout/commercial source access or human playback was used; this is not an audible-effect or TUI-polish claim.
