# `RIOTBOX-1547` Replace final Feral test includes with explicit regression and synthetic fixture modules

- Ticket: `RIOTBOX-1547`
- Title: `Replace final Feral test includes with explicit regression and synthetic fixture modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1547/replace-final-feral-test-includes-with-explicit-regression-and`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1547-feral-test-module-ownership`
- Linear branch: `feature/riotbox-1547-replace-final-feral-test-includes-with-explicit-regression`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1638 (https://github.com/marang/riotbox/pull/1638)`
- Merge commit: `7e609fb6d1cb17eccf8dc3a3b320a846e4c8b15f`
- Deleted from Linear: `2026-10-01`
- Verification: `Fresh same 143 Debug/Release names/statuses; twelve complete test/attribute bodies equal under identical rustfmt normalization and four fixture bodies preserved; main/production contracts unchanged. Fresh five-binary build; actual 141 CLI/85 artifact equality, alias preservation and both comparator manifests. Warning-free full source-free just ci and solo Rust/design/spec/evidence review; native 36857054098 both Rust/Windows SUCCESS exact c0049b9bdcf0359d77e7eaaf9bd1f1cd3d2312cd, fresh reviews empty; whole candidate equals merged main.`
- Docs touched: `docs/engineering/module_policy.md; docs/engineering/textual_include_allowlist.txt; docs/engineering/textual_include_inventory_2026-06-29.md; docs/research_decision_log.md; docs/reviews/riotbox_1547_feral_test_module_ownership_2026-10-01.md`
- Follow-ups: `RIOTBOX-1548: actual remaining manifest/test root aliases. RIOTBOX-1509: fresh minimized Windows repro still required.`

## Why This Ticket Existed

Remove the final P023 QA test lexical capture while retaining all existing regression identities and synthetic controls.

## What Shipped

- Two ordinary cfg(test) owners retain twelve tests under tests:: and all four original fixtures; explicit actual-owner imports replace wildcard root capture. Exactly 26 obsolete root imports removed; zero include sites with existing empty allowlist, RBX-415/policy/inventory/report.

## Notes

- Identical rustfmt normalization explicitly accounts for one expression-closure brace pair. Zero includes is not a thin facade. No real sources, holdouts, device/DAW/human playback or subagents; no musical/source/hardness/release verdict.
