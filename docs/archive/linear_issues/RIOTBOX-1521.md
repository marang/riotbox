# `RIOTBOX-1521` Isolate Feral grid numerical evidence and grid/filter dependency leaves

- Ticket: `RIOTBOX-1521`
- Title: `Isolate Feral grid numerical evidence and grid/filter dependency leaves`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1521/isolate-feral-grid-numerical-evidence-and-gridfilter-dependency-leaves`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1521-feral-grid-metric-owners`
- Linear branch: `feature/riotbox-1521-isolate-feral-grid-numerical-evidence-and-gridfilter`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#1586 (https://github.com/marang/riotbox/pull/1586)`
- Merge commit: `9b7da7fca0d2482abb52904ac86e561c6bab131f`
- Deleted from Linear: `2026-10-01`
- Verification: `73 complete definitions exact; 44 Debug/Release synthetic test leaves/statuses retained with four documented drift module-path moves; 31 exact CLI cases; 39 exact hashes including 16 pack WAVs plus input; both manifests validate; full source-free just ci/final logs warning-error-free; native run 36805268552 both jobs SUCCESS, zero outstanding reviews/comments.`
- Docs touched: `docs/reviews/riotbox_1521_feral_grid_metric_owners_2026-10-01.md`, `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1522: demonstrated verification-command quoting defect, separate behavioral fix; RIOTBOX-1509 remains open.`

## Why This Ticket Existed

Numerical QA metrics depended lexically on orchestration Grid and render-stem filtering.

## What Shipped

- Ordinary bar/spectral/drift owners and drift test child with unchanged config/Grid/frame/filter leaves. Explicit compatibility imports preserve legacy consumers; RBX-395; includes 30/2 to 26/2, not a whole root migration.

## Notes

- Solo review/self-review zero remaining migration findings after traced test path/cfg import correction, no suppression. No real sources, directory discovery, device/DAW/playback or human/source-general qualification. Cadence four since RIOTBOX-1516. Archive native gates and actual cleanup tracked separately.
