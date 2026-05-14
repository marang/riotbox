# `RIOTBOX-797` Add syncopated source showcase smoke gate

- Ticket: `RIOTBOX-797`
- Title: `Add syncopated source showcase smoke gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-797/add-syncopated-source-showcase-smoke-gate`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-797-syncopated-showcase-smoke`
- Linear branch: `feature/riotbox-797-add-syncopated-source-showcase-smoke-gate`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#792 (https://github.com/marang/riotbox/pull/792)`
- Merge commit: `7819168994c2ef7147e3a215eb5a6ffd74098fd0`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-797-syncopated-smoke.log just syncopated-source-showcase-smoke`; `scripts/run_compact.sh /tmp/riotbox-797-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-797-just-list.log just --list`; `git diff --check`; `GitHub Actions Rust CI run 1915 passed on 332ca055dc42a86527f6153403584157c1833d81`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-795 showed that the full representative source showcase caught a syncopated-source Source Timing regression that normal CI did not exercise.

## What Shipped

- Added a bounded syncopated source showcase smoke that generates deterministic sources, runs feral_grid_pack for the syncopated-snare source, validates the listening manifest/artifacts, and gates source timing, source-grid drift, TR-909/W-30 alignment, loop closure, and non-silent output metrics through audio-qa-ci.

## Notes

- The full representative showcase remains a local review/listening pack; this is the compact CI-safe regression path only.
