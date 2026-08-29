# `RIOTBOX-1486` Quantize the real live master capture to an exact two-bar Session window

- Ticket: `RIOTBOX-1486`
- Title: `Quantize the real live master capture to an exact two-bar Session window`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1486/quantize-the-real-live-master-capture-to-an-exact-two-bar-session`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-29`
- Started: `2026-08-29`
- Finished: `2026-08-29`
- Branch: `feature/riotbox-1486-quantize-the-real-live-master-capture-to-an-exact-two-bar`
- Linear branch: `feature/riotbox-1486-quantize-the-real-live-master-capture-to-an-exact-two-bar`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1506 (https://github.com/marang/riotbox/pull/1506)`
- Merge commit: `647c93bcda0be256f376c84a8ae8f3c253bbb940`
- Deleted from Linear: `2026-08-29`
- Verification: `focused callback, Core, app, observer, CLI, and contract-smoke tests passed; cargo fmt, diff check, strict app/core/audio clippy, and full just ci passed; formal code/Rust review fixed four findings and repeat review found zero; GitHub rust-ci passed; final real ALSA user-session capture requested beat 39 from confirmed anchor 3, captured beats 39.000009 through 47.000027 as 160364 stereo float32 frames at 44.1 kHz and 132 BPM, with all capture fault counters zero`
- Docs touched: `README.md, docs/execution_roadmap.md, docs/research_decision_log.md, docs/specs/action_lexicon_spec.md, docs/specs/audio_core_spec.md, docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1036 remains the P016 backlog anchor for broader full-arrangement, stem, and live-recording export workflows.`

## Why This Ticket Existed

RIOTBOX-1485 recorded a bounded live callback window, but it began at an arbitrary callback edge rather than the confirmed Session bar phase and therefore could not guarantee a clean musician-facing two-bar export window.

## What Shipped

- Added the versioned RuntimeMasterBarWindowV2 action/product boundary while preserving V1 receipt readability and behavior.
- Armed the preallocated realtime capture against the exact confirmed Session bar-grid anchor, handled straddling callbacks, and failed closed on skipped or contradictory timing.
- Published requested and captured beat geometry plus fault counters through proof, Session receipt, replay, observer, CLI, and readiness validation.

## Notes

- Branch review identified and fixed incorrect zero-phase bar math, incomplete armed-wait callback-gap accounting, accidental V1 callback-semantic drift, and insufficient receipt timing cross-validation; the repeat review found no surviving findings.
- The final host run validated timing and transport preservation only. No DSP, musical-quality, source-general, Holdout, commercial-reference, or human-listening claim was made.
