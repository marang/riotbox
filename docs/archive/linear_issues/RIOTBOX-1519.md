# `RIOTBOX-1519` Replace W-30 preview QA includes with semantic Rust module owners

- Ticket: `RIOTBOX-1519`
- Title: `Replace W-30 preview QA includes with semantic Rust module owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1519/replace-w-30-preview-qa-includes-with-semantic-rust-module-owners`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1519-semantic-w30-preview`
- Linear branch: `feature/riotbox-1519-replace-w-30-preview-qa-includes-with-semantic-rust-module`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#1582 (https://github.com/marang/riotbox/pull/1582)`
- Merge commit: `7f3b0cb4a8d978b2d080e5df93e1ebd584128e8e`
- Deleted from Linear: `2026-10-01`
- Verification: `93 definitions exact after visibility/trailing-comma normalization; 27 unchanged Debug/Release test leaves; 42 exact CLI cases; 24 exact fresh synthetic artifacts; warning/error-free source-free just ci; native run 36802777370 both jobs SUCCESS, zero outstanding comments/reviews.`
- Docs touched: `docs/reviews/riotbox_1519_semantic_w30_preview_2026-10-01.md`, `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1520: independent Feral before/after ownership slice; RIOTBOX-1509 remains open.`

## Why This Ticket Existed

W-30 QA rendering/input adaptation and metric policy/manifest presentation shared six lexical includes.

## What Shipped

- Same two Cargo binaries with private semantic owners, complete algorithms and regressions preserved; includes 38/5 to 32/3. RBX-393 and canonical module policy/inventory updated.

## Notes

- Solo review and self-review: zero additional changed-diff findings. Maintenance only; no real sources, source directory search, device, DAW, playback or human verdict. Architecture cadence two since RIOTBOX-1516. Archive native gates and actual deletion tracked separately.
