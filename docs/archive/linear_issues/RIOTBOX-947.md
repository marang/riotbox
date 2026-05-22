# `RIOTBOX-947` Add downbeat ambiguity compatibility to generated Feral-grid summary index

- Ticket: `RIOTBOX-947`
- Title: `Add downbeat ambiguity compatibility to generated Feral-grid summary index`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-947/add-downbeat-ambiguity-compatibility-to-generated-feral-grid-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-947-add-downbeat-ambiguity-compatibility-to-generated-feral-grid`
- Linear branch: `feature/riotbox-947-add-downbeat-ambiguity-compatibility-to-generated-feral-grid`
- Assignee: `Markus`
- Labels: None
- PR: `#940 (https://github.com/marang/riotbox/pull/940)`
- Merge commit: `42029e3314585a09216e7cbbdd744606abcf1220`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-generated-feral-grid`; `git diff --check`; `just ci`; `GitHub Actions Rust CI run 26299475241 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The generated Feral-grid observer/audio gate already validated downbeat ambiguity compatibility in JSON, but the compact summary.tsv index only exposed broad alignment, forcing reviewers to reopen JSON to distinguish partial ambiguity evidence from locked-grid alignment.

## What Shipped

- Added a downbeat_ambiguity column to artifacts/audio_qa/local/generated-feral-grid-observer-audio/summary.tsv.
- Populated the column from output_path.source_timing_alignment.downbeat_ambiguity_compatibility.
- Pinned TSV greps for cautious/manual, fallback, and locked-grid rows so partial vs aligned ambiguity compatibility stays visible.

## Notes

- No analyzer, ActionCommand, Session/replay, JamAppState, realtime audio, or generated audio behavior changed.
