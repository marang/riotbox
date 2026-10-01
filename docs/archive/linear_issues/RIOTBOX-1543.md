# `RIOTBOX-1543` Replace Feral pack orchestration include with explicit renderer and report ownership

- Ticket: `RIOTBOX-1543`
- Title: `Replace Feral pack orchestration include with explicit renderer and report ownership`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1543/replace-feral-pack-orchestration-include-with-explicit-renderer-and`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1543-feral-pack-orchestration-owner`
- Linear branch: `feature/riotbox-1543-replace-feral-pack-orchestration-include-with-explicit`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1630 (https://github.com/marang/riotbox/pull/1630)`
- Merge commit: `5c9081386c47ff34c10232d37726bba9b0869842`
- Deleted from Linear: `2026-10-01`
- Verification: `143 same Debug/Release tests; normalized token identity; 141 actual CLI records and 85 artifact hashes identical; full source-free just ci; native 36849029377 both jobs green for exact d03787c3ae8300b3c5e198730891d0d05dd50aa0; whole candidate tree equals merged main.`
- Docs touched: `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1543_feral_pack_orchestration_2026-10-01.md`
- Follow-ups: `RIOTBOX-1544: separate Markdown publication owner. Three root legacy includes and the test helper remain. P023 source/human/DAW gates unchanged.`

## Why This Ticket Existed

P023 QA artifact review required explicit Feral orchestration/report ownership instead of lexical capture.

## What Shipped

- Root CLI main, unchanged renderer/source-format owner and shared 31-field offline report value use real binary-private modules; include count five to four.

## Notes

- Generated controls/retained fixtures only; solo review, no real sources/Holdout/commercial audio, playback, DSP/policy/schema/runtime/Core/Session changes or musical/hardness/human/release claim.
