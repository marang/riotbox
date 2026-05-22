# `RIOTBOX-931` Validate Source Timing downbeat ambiguity observer fields

- Ticket: `RIOTBOX-931`
- Title: `Validate Source Timing downbeat ambiguity observer fields`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-931/validate-source-timing-downbeat-ambiguity-observer-fields`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-931-validate-downbeat-ambiguity-observer`
- Linear branch: `feature/riotbox-931-validate-source-timing-downbeat-ambiguity-observer-fields`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`
- PR: `#924 (https://github.com/marang/riotbox/pull/924)`
- Merge commit: `2ed2c8bd1e1d48fb6eb2cfe8ae9e33b4d174907b`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; GitHub Actions Rust CI run 26292017355 passed`
- Docs touched: `n/a`
- Follow-ups: `RIOTBOX-932 covers valid observer summary rendering for the same fields.`

## Why This Ticket Existed

Make RIOTBOX-930 downbeat ambiguity observer fields QA-enforced when malformed values are present.

## What Shipped

- Added strict observer/audio correlation tests rejecting malformed primary downbeat score, score gap, and alternate downbeat count shapes.

## Notes

- Fields remain optional for older observer fixtures; no analyzer, UI, Session, ActionCommand, or audio-output behavior changed.
