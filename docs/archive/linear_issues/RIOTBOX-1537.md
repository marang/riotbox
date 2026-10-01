# `RIOTBOX-1537` Reject Before/After pack destinations that replace their input source

- Ticket: `RIOTBOX-1537`
- Title: `Reject Before/After pack destinations that replace their input source`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1537/reject-beforeafter-pack-destinations-that-replace-their-input-source`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1537-before-after-source-output-collision`
- Linear branch: `feature/riotbox-1537-reject-beforeafter-pack-destinations-that-replace-their`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1618 (https://github.com/marang/riotbox/pull/1618)`
- Merge commit: `468a9e9e9ca4d08bca76bedfc5372120424a5588`
- Deleted from Linear: `2026-10-01`
- Verification: `94 identical Debug/Release names and passing statuses; 31 identical CLI records; 29 identical artifact/input SHA-256 values; original source-loss loop rejects without mutation; source-free just ci and native exact-head Rust/Windows-sidecar checks green`
- Docs touched: `RBX-407; Audio QA Manifests And Artifacts; docs/reviews/riotbox_1537_before_after_source_preservation_2026-10-01.md`
- Follow-ups: `RIOTBOX-1538 independently addresses confirmed W-30 comparator input/report/manifest collisions; no Stage-A, DAW, device or human-review claim`

## Why This Ticket Existed

Before/After QA rendering must not destroy its registered input through any output path

## What Shipped

- One typed 14-destination layout drives shared physical-identity preflight and artifact writers before the first source-excerpt write

## Notes

- Stable namespace only; no atomic pack or concurrent attacker claim; no real audio sources or subagents; archive publication and branch/Linear cleanup verified separately
