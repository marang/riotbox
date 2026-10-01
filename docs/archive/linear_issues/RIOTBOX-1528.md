# `RIOTBOX-1528` Separate Feral TR-909 QA profile/pressure policy from presentation and root rendering

- Ticket: `RIOTBOX-1528`
- Title: `Separate Feral TR-909 QA profile/pressure policy from presentation and root rendering`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1528/separate-feral-tr-909-qa-profilepressure-policy-from-presentation-and`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1528-feral-tr909-qa-owners`
- Linear branch: `feature/riotbox-1528-separate-feral-tr-909-qa-profilepressure-policy-from`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1600 (https://github.com/marang/riotbox/pull/1600)`
- Merge commit: `7dc792487c36c4dd33d5f7a98ed317ded4a1af0f`
- Deleted from Linear: `2026-10-01`
- Verification: `All 76 complete definitions unchanged after visibility/trailing-comma normalization; same 54 Debug/Release test identities, 31 CLI outcomes and 39 hashes including 16 synthetic output WAVs; two manifests validate artifacts; final source-free just ci green; native PR1600 Rust and Windows Sidecar jobs green, zero reviews/comments; P3 import findings fixed, final solo review/self-review zero open.`
- Docs touched: `docs/engineering/module_policy.md; docs/engineering/textual_include_inventory_2026-06-29.md; docs/engineering/textual_include_allowlist.txt; docs/research_decision_log.md; docs/reviews/riotbox_1528_feral_tr909_owners_2026-10-01.md`
- Follow-ups: `RIOTBOX-1529 bounded QA source-window owner; RIOTBOX-1509 remains Todo awaiting fresh causal startup evidence.`

## Why This Ticket Existed

Remove backwards root render/frame dependencies and mixed TR-909 QA policy/serializer ownership without changing behavior.

## What Shipped

- Ordinary source-profile and kick/accent pressure owners, separate three manifest DTO/conversion pairs, existing fractional Grid frame helper and primitive render seam ownership; all regressions retained; RBX-399; include inventory 16 to 13.

## Notes

- No independent panel, real source, holdout, commercial audio, device, DAW, playback, musical/hardness/source-general claim. Existing QA wrapper dead-code attributes retained, no new suppression. Structural cadence two since RIOTBOX-1524 after implementation merge.
