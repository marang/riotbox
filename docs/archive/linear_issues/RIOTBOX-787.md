# `RIOTBOX-787` Add locked-grid observer/audio correlation fixture pair for Source Timing alignment

- Ticket: `RIOTBOX-787`
- Title: `Add locked-grid observer/audio correlation fixture pair for Source Timing alignment`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-787/add-locked-grid-observeraudio-correlation-fixture-pair-for-source`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-787-locked-observer-audio-correlation`
- Linear branch: `feature/riotbox-787-add-locked-grid-observeraudio-correlation-fixture-pair-for`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#782 (https://github.com/marang/riotbox/pull/782)`
- Merge commit: `52cdfbad6b321f1a5beb4853c8889fc13d20aa57`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-787-rebase-locked-grid-fixture.log just observer-audio-correlate-locked-grid-json-fixture`; `scripts/run_compact.sh /tmp/riotbox-787-rebase-summary-validator.log just observer-audio-summary-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-787-rebase-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-787-rebase-fmt.log cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed a committed observer/audio fixture pair proving locked app-observed Source Timing and locked manifest Source Timing align at the strict correlation surface.

## What Shipped

- Added locked-grid observer, manifest, and summary fixtures; added a strict observer/audio locked-grid JSON fixture target; wired it into audio-qa-ci and documented the gate.

## Notes

- No detector thresholds, lane sound design, ActionCommand, JamAppState, or production audio rendering behavior changed.
