# `RIOTBOX-789` Add MC-202 lane recipe summary validator negative fixtures

- Ticket: `RIOTBOX-789`
- Title: `Add MC-202 lane recipe summary validator negative fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-789/add-mc-202-lane-recipe-summary-validator-negative-fixtures`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-789-lane-recipe-summary-negative-fixtures`
- Linear branch: `feature/riotbox-789-add-mc-202-lane-recipe-summary-validator-negative-fixtures`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#784 (https://github.com/marang/riotbox/pull/784)`
- Merge commit: `8a940f681f91c7737e7c332fd5835ca5f8e14fba`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-789-rebase-summary-validator.log just observer-audio-summary-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-789-rebase-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-789-rebase-fmt.log cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

After RIOTBOX-788 exposed lane_recipe_cases in observer/audio summaries, P012 needed negative fixtures proving malformed MC-202 phrase-grid and source-phrase-slot evidence is rejected.

## What Shipped

- Added invalid observer/audio summary fixtures for malformed lane recipe phrase-grid and source-phrase-slot evidence and wired both into the summary validator fixture recipe.

## Notes

- No audio generation, ActionCommand, app runtime, or lane metric behavior changed.
