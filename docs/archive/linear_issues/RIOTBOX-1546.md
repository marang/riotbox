# `RIOTBOX-1546` Replace mixed Feral stem include with explicit rendering validation and artifact owners

- Ticket: `RIOTBOX-1546`
- Title: `Replace mixed Feral stem include with explicit rendering validation and artifact owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1546/replace-mixed-feral-stem-include-with-explicit-rendering-validation`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1546-feral-stem-semantic-owners`
- Linear branch: `feature/riotbox-1546-replace-mixed-feral-stem-include-with-explicit-rendering`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1636 (https://github.com/marang/riotbox/pull/1636)`
- Merge commit: `4e18704e4be5398dbba777dfa4ba12ffedc21866`
- Deleted from Linear: `2026-10-01`
- Verification: `Fresh same 143 Debug/Release names/statuses, thirteen whole-function token matches plus root main/builder/manifest definitions; fresh five-binary build; all 141 parsed CLI records and 85 artifact hashes equal accepted original baselines; fresh alias rejection preserves inputs/prior outputs; both comparator manifests validate; warning-free full source-free just ci, formatting/diff/include and solo Rust/design/spec/evidence review. Native 36855321585 both Rust/Windows SUCCESS exact e11151befff15999cc79805d616ac0d739205f43; fresh comments/reviews empty; whole candidate tree equals merged main.`
- Docs touched: `docs/engineering/module_policy.md; docs/engineering/textual_include_allowlist.txt; docs/engineering/textual_include_inventory_2026-06-29.md; docs/research_decision_log.md; docs/reviews/riotbox_1546_feral_stem_semantic_owners_2026-10-01.md`
- Follow-ups: `RIOTBOX-1547: remaining test/helper include ownership; RIOTBOX-1509: fresh minimized Windows repro still required.`

## Why This Ticket Existed

Remove root lexical coupling of rendering, validation, artifact publication and reproduction commands from P023 QA review.

## What Shipped

- Four explicit offline owners replace mixed stem lexical capture; renderer and manifest use actual sibling interfaces; thirteen complete function/attribute bodies and legacy test identities retained. RBX-414, module policy, include inventory 3→2 and durable review evidence shipped.

## Notes

- Mechanical namespace change only; no real sources, holdouts, DAW/device/playback, subagents or musical/source/hardness/human/release verdict. Root is not a thin facade.
