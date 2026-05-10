# `RIOTBOX-267` Refresh footer screenshot baselines for compact key legend

- Ticket: `RIOTBOX-267`
- Title: `Refresh footer screenshot baselines for compact key legend`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-267/refresh-footer-screenshot-baselines-for-compact-key-legend`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-267-refresh-footer-screenshot-baselines-for-compact-key-legend`
- Linear branch: `feature/riotbox-267-refresh-footer-screenshot-baselines-for-compact-key-legend`
- PR: `#257`
- Merge commit: `cedb2ff`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-268`

## Why This Ticket Existed

`RIOTBOX-266` compressed the always-visible Jam footer key legend. Several broad screenshot baselines still quoted the older long-form `Keys:` footer text, which made those repo docs drift from the current shell surface.

## What Shipped

- Refreshed the broad screenshot baseline footer `Keys:` lines for the compact `1-4 screens` legend.
- Preserved the existing artifact widths.
- Left older narrow artifacts unchanged because their footer lines already predate the current full footer surface.

## Verification

- `git diff --check`
- `rg -n "1-4 screens|i inspect|r re-ingest" docs/screenshots/capture_screen_baseline.txt docs/screenshots/capture_w30_live_recall_baseline.txt docs/screenshots/jam_log_screen_baseline.txt docs/screenshots/source_screen_baseline.txt`
- `rg -n "1 jam|2 log|3 source|4 capture|re-ingest source" docs/screenshots/capture_screen_baseline.txt docs/screenshots/capture_w30_live_recall_baseline.txt docs/screenshots/jam_log_screen_baseline.txt docs/screenshots/source_screen_baseline.txt || true`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no UI behavior, keymap behavior, screenshot tooling, or broad benchmark rewrite changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
