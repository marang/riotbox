# `RIOTBOX-938` Assert downbeat ambiguity compatibility in generated Feral-grid gates

- Ticket: `RIOTBOX-938`
- Title: `Assert downbeat ambiguity compatibility in generated Feral-grid gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-938/assert-downbeat-ambiguity-compatibility-in-generated-feral-grid-gates`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-938-generated-downbeat-ambiguity-gates`
- Linear branch: `feature/riotbox-938-assert-downbeat-ambiguity-compatibility-in-generated-feral`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#931 (https://github.com/marang/riotbox/pull/931)`
- Merge commit: `aa0c2ef6292c73d7b131cdcb0237e8a580b7fce7`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-locked-grid-json-fixture; just observer-audio-correlate-generated-feral-grid; just audio-qa-ci; just ci; git diff --check; GitHub Actions Rust CI run 26294827978 passed`
- Docs touched: `n/a`
- Follow-ups: `Continue P012 source-timing spine with the next smallest roadmap-aligned proof.`

## Why This Ticket Existed

Assert downbeat ambiguity compatibility in generated Feral-grid observer/audio gates so the all-lane QA path proves the field explicitly.

## What Shipped

- Added jq assertions for source_timing_alignment.downbeat_ambiguity_compatibility across cautious/manual-confirm, user-override, risky override, fallback/unavailable, and locked-grid generated Feral-grid summaries, plus the committed locked-grid JSON fixture smoke.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, realtime audio, or generated audio behavior changed.
