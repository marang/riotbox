# `RIOTBOX-1513` Replace TR-909 Core policy includes with explicit semantic policy modules

- Ticket: `RIOTBOX-1513`
- Title: `Replace TR-909 Core policy includes with explicit semantic policy modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1513/replace-tr-909-core-policy-includes-with-explicit-semantic-policy`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1513-semantic-tr909-policy`
- Linear branch: `feature/riotbox-1513-replace-tr-909-core-policy-includes-with-explicit-semantic`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1570 (https://github.com/marang/riotbox/pull/1570)`
- Merge commit: `96adfb322530d59560a26da01e2303a2ef3bb387`
- Deleted from Linear: `2026-10-01`
- Verification: `All 29 production definitions retained; strict 17 model/impl comparison; ten test/helper definitions including five regressions unchanged; identical 470 Core leaf/status tests pass; warning-free explicitly scanned Core/App/Audio check; full source-free just ci and strict Clippy green; sequential solo Rust/self-review no unresolved findings; native exact-head run 36793582209 both jobs green, no reviews/comments.`
- Docs touched: `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1513_semantic_tr909_policy_2026-10-01.md`
- Follow-ups: `RIOTBOX-1514: observer/audio correlation semantic diagnostic ownership; mandatory architecture checkpoint after that fifth substantive migration.`

## Why This Ticket Existed

Make the existing Core TR-909 policy reviewable through semantic Rust owners instead of a shared textual include namespace.

## What Shipped

- Ordinary private model, source-support, render-composition, pattern/phrase and regression modules; explicit compatibility facade preserves nine public types and three functions; include inventory 56/8 to 54/7.

## Notes

- RBX-390 records ownership only. Fourth substantive architecture slice since RIOTBOX-1507; test-only/archive work excluded. No schema, policy, threshold, algorithm or audio change; no real source/holdout/reference/device/DAW/playback access. Initial missing test enum import corrected narrowly before replacement checks. No subagents.
