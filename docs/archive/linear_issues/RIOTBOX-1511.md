# `RIOTBOX-1511` Replace Audio runtime regression include namespace with semantic test modules

- Ticket: `RIOTBOX-1511`
- Title: `Replace Audio runtime regression include namespace with semantic test modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1511/replace-audio-runtime-regression-include-namespace-with-semantic-test`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1511-semantic-runtime-tests`
- Linear branch: `feature/riotbox-1511-replace-audio-runtime-regression-include-namespace-with`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1566 (https://github.com/marang/riotbox/pull/1566)`
- Merge commit: `0c92d9814197499846c6c151450f2b7bc609406d`
- Deleted from Linear: `2026-10-01`
- Verification: `Full source-free just ci; warning-free all-target Audio check; Audio baseline/post identical 280 leaf/status entries (279 pass, same ignored benchmark); all 174 definitions and 129 selected tests retained; include guard 58/9; solo code-review/Rust/self-review no unresolved findings; exact-head native Rust CI and Windows Sidecar run 36785701086 both passed.`
- Docs touched: `docs/engineering/module_policy.md, docs/engineering/textual_include_allowlist.txt, docs/engineering/textual_include_inventory_2026-06-29.md, docs/research_decision_log.md, docs/reviews/riotbox_1511_semantic_runtime_tests_2026-10-01.md`
- Follow-ups: `RIOTBOX-1512 JamApp lane-projection module migration; RIOTBOX-1509 remains Todo with no causal Windows startup fix claim.`

## Why This Ticket Existed

Replace twelve synthetic Runtime regression includes and hidden fixture imports with explicit semantic test ownership without changing production audio behavior.

## What Shipped

- Ordinary lifecycle/shared-state/mix/lane/transport-stop/Source Monitor/fill/gesture/metrics test modules and bounded shared fixtures; direct telemetry-test imports; no production runtime/DSP/API change.

## Notes

- RBX-388 freezes test ownership only. No real source/holdout/reference/device/DAW/playback or musical claim. Test-only migration excludes substantive architecture cadence: remains two since RIOTBOX-1507.
