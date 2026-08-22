# `RIOTBOX-1410` Make source-monitor updates replace source ownership explicitly

- Ticket: `RIOTBOX-1410`
- Title: `Make source-monitor updates replace source ownership explicitly`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1410/make-source-monitor-updates-replace-source-ownership-explicitly`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-08-22`
- Finished: `2026-08-22`
- Branch: `feature/riotbox-1410-make-source-monitor-updates-replace-source-ownership`
- Linear branch: `feature/riotbox-1410-make-source-monitor-updates-replace-source-ownership`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `review-followup`
- PR: `#1428 (https://github.com/marang/riotbox/pull/1428)`
- Merge commit: `09001a04c06715434aa214702465b2a4099023b7`
- Deleted from Linear: `2026-08-22`
- Verification: `Full just ci passed locally.`; `GitHub Rust CI passed on PR #1428.`; `Focused riotbox-audio and riotbox-app tests, Clippy -D warnings, formatting, diff checks, and Rust/code review passed with no remaining findings.`
- Docs touched: `docs/README.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, `docs/research_decision_log.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: `RIOTBOX-1407 next, followed by RIOTBOX-1334, to complete the remaining named Foundation gaps.`

## Why This Ticket Existed

Prevent source-monitor selection/control state from diverging from callback-visible prepared PCM during live source replacement.

## What Shipped

- Published prepared PCM metadata, mode-derived gains, and timing anchors as one immutable atomic source-monitor snapshot.
- Separated control-only updates from explicit complete source replacement and made missing replacement fail closed.
- Added callback-safe control-side retirement so the realtime callback cannot become the last owner that deallocates large PCM.

## Notes

- No Development/Holdout/commercial source audio or human listening was used or claimed.
