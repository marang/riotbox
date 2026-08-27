# `RIOTBOX-1479` Commit source-matched stem handoff through the Session export action

- Ticket: `RIOTBOX-1479`
- Title: `Commit source-matched stem handoff through the Session export action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1479/commit-source-matched-stem-handoff-through-the-session-export-action`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-26`
- Started: `2026-08-26`
- Finished: `2026-08-27`
- Branch: `feature/riotbox-1479-commit-source-matched-stem-handoff-through-session-export-action`
- Linear branch: `feature/riotbox-1479-commit-source-matched-stem-handoff-through-the-session`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1492 (https://github.com/marang/riotbox/pull/1492)`
- Merge commit: `160bda015ac7948374e5e7cccb30314bb3f781c8`
- Deleted from Linear: `2026-08-27`
- Verification: `Local source-free just ci and strict all-target Clippy passed; focused handoff/CLI tests passed; GitHub rust-ci passed in PR #1492.`
- Docs touched: `README.md; docs/README.md; docs/execution_roadmap.md; docs/specs/action_lexicon_spec.md; docs/specs/audio_core_spec.md; docs/specs/audio_qa/manifests_and_artifacts.md; docs/specs/session_file_spec.md; docs/research_decision_log.md; docs/reviews/riotbox_1479_source_matched_stem_session_handoff_2026-08-27.md`
- Follow-ups: `Musician-facing interaction, DAW placement, and structured listening remain intentionally disabled and require a separate Linear-first slice.`

## Why This Ticket Existed

Connect the validated source-matched v2 product-stem handoff to the canonical Action, Session, replay, observer, and export side-effect path before exposing musician controls.

## What Shipped

- Added strict typed v2 handoff parsing and validation with exact active Source Graph and Session graph identity.
- Published only validated Drums, Music, and Bass WAV bytes through staging and atomic no-clobber commit, with reconstruction and lineage proof.
- Committed the existing stem-package export action, Session receipt, timing lineage, and Observer lifecycle; all invalid evidence fails closed without partial output.

## Notes

- No registered Development source, Holdout, commercial reference, source-directory discovery, playback, or human listening verdict was used.
