# `RIOTBOX-933` Expose manifest downbeat ambiguity evidence in observer/audio summary

- Ticket: `RIOTBOX-933`
- Title: `Expose manifest downbeat ambiguity evidence in observer/audio summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-933/expose-manifest-downbeat-ambiguity-evidence-in-observeraudio-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-933-manifest-downbeat-ambiguity-summary`
- Linear branch: `feature/riotbox-933-expose-manifest-downbeat-ambiguity-evidence-in-observeraudio`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`
- PR: `#926 (https://github.com/marang/riotbox/pull/926)`
- Merge commit: `d44584361fda15c48942635365315e3bfe1eb00d`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; GitHub Actions Rust CI run 26292478895 passed`
- Docs touched: `n/a`
- Follow-ups: `RIOTBOX-934 validates malformed manifest-side downbeat ambiguity fields.`

## Why This Ticket Existed

Expose output-path manifest downbeat ambiguity score, margin, and alternate count in reviewer-facing observer/audio summaries.

## What Shipped

- Parsed optional manifest-side downbeat ambiguity fields and rendered them in Markdown and JSON source_timing summaries with focused smoke coverage.

## Notes

- No analyzer, UI, Session, ActionCommand, or audio-output behavior changed.
