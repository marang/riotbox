# `RIOTBOX-907` Document external review freshness and current Riotbox source assessment

- Ticket: `RIOTBOX-907`
- Title: `Document external review freshness and current Riotbox source assessment`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-907/document-external-review-freshness-and-current-riotbox-source`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-907-external-review-refresh`
- Linear branch: `feature/riotbox-907-document-external-review-freshness-and-current-riotbox`
- Assignee: `Markus`
- Labels: `Docs`, `review-followup`, `workflow`
- PR: `#900 (https://github.com/marang/riotbox/pull/900)`
- Merge commit: `2357cb607fe328d453138ea5bc061f9f1030d727`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; git diff --check; scripts/run_compact.sh /tmp/riotbox-907-ci.log just ci; GitHub Rust CI #2243 passed`
- Docs touched: `docs/reviews/external_review_refresh_2026-05-22.md; docs/README.md; docs/reviews/README.md; docs/workflow_conventions.md`
- Follow-ups: `RIOTBOX-908 converts the audio runtime include shell; RIOTBOX-909 audits the TUI include shell later`

## Why This Ticket Existed

Preserve useful external review signal while freshness-checking stale or incomplete source-level findings before they become backlog work.

## What Shipped

- Added an external review refresh note, linked it from docs indexes, and added workflow guidance requiring external review findings to be verified against current main, Linear, and review docs before ticket creation.

## Notes

- Docs/workflow-only slice; no product, runtime, ActionCommand, Session/replay, JamAppState, lane, or audio-output behavior changed.
