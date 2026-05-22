# `RIOTBOX-935` Compare observer and manifest downbeat ambiguity evidence

- Ticket: `RIOTBOX-935`
- Title: `Compare observer and manifest downbeat ambiguity evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-935/compare-observer-and-manifest-downbeat-ambiguity-evidence`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-935-downbeat-ambiguity-alignment`
- Linear branch: `feature/riotbox-935-compare-observer-and-manifest-downbeat-ambiguity-evidence`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`
- PR: `#928 (https://github.com/marang/riotbox/pull/928)`
- Merge commit: `ca43140b110b7e07a2d88352827dbb65b9e3ef4c`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; just observer-audio-correlate-generated-feral-grid; GitHub Actions Rust CI run 26293604394 passed`
- Docs touched: `n/a`
- Follow-ups: `RIOTBOX-936 pins Beat20 real-source downbeat ambiguity proof fields.`

## Why This Ticket Existed

Compare observer and manifest downbeat ambiguity evidence so QA catches disagreement about manual-confirm reasons without treating one-sided evidence as a hard contradiction.

## What Shipped

- Added downbeat_ambiguity_compatibility to source timing alignment, Markdown/JSON summary rendering, strict alternate-count mismatch coverage, and a partial-evidence fix for older observer fields.

## Notes

- No analyzer, UI, Session, ActionCommand, or audio-output behavior changed. First CI run exposed an overly strict partial-evidence failure; follow-up commit f2f13a20 fixed it and rerun passed.
