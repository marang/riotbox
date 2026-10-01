# `RIOTBOX-1520` Replace Feral before/after QA includes with semantic diagnostic owners

- Ticket: `RIOTBOX-1520`
- Title: `Replace Feral before/after QA includes with semantic diagnostic owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1520/replace-feral-beforeafter-qa-includes-with-semantic-diagnostic-owners`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1520-semantic-feral-before-after`
- Linear branch: `feature/riotbox-1520-replace-feral-beforeafter-qa-includes-with-semantic`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#1584 (https://github.com/marang/riotbox/pull/1584)`
- Merge commit: `e8a7db660c060060b3835993c6a086cca9175076`
- Deleted from Linear: `2026-10-01`
- Verification: `41 complete definitions exact; three unchanged Debug/Release test leaves; 29 exact CLI results; 28 exact synthetic pack hashes including 12 WAVs plus unchanged input hash; both manifests validate; full source-free just ci warning/error-free; exact-head native run 36803787105 both jobs SUCCESS, zero outstanding reviews/comments.`
- Docs touched: `docs/reviews/riotbox_1520_semantic_feral_before_after_2026-10-01.md`, `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1521: bounded Feral grid numerical-evidence ownership; RIOTBOX-1509 remains open.`

## Why This Ticket Existed

Feral before-after diagnostic source/render/mix/metadata shared two mechanical include shards.

## What Shipped

- Same Cargo binary with semantic private CLI/config, orchestration, source-window, fixed render plan, mix/delta, artifact I/O, Markdown and manifest owners. RBX-394; includes 32/3 to 30/2, preserving behavior.

## Notes

- Solo review and self-review zero changed-diff findings; not an independent panel. No real sources, directory discovery, devices, DAW/playback or human qualification. Existing side-effect order and CLI quirks deliberately unchanged. Cadence three since RIOTBOX-1516. Actual archive gates and deletion tracked separately.
