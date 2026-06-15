# `RIOTBOX-1276` MC-202 production sound-design pass for source-composed motifs

- Ticket: `RIOTBOX-1276`
- Title: `MC-202 production sound-design pass for source-composed motifs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1276/mc-202-production-sound-design-pass-for-source-composed-motifs`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1276-mc202-production-sound-design`
- Linear branch: `feature/riotbox-1276-mc-202-production-sound-design-pass-for-source-composed`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1251 (https://github.com/marang/riotbox/pull/1251)`
- Merge commit: `48455f6a`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-audio mc202 -- --nocapture; cargo test -p riotbox-app committed_mc202_answer -- --nocapture; just professional-output-listening-pack-smoke (/tmp/riotbox-1276-professional-output-listening-pack-smoke.log); git diff --check; just ci (/tmp/riotbox-1276-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1277 should tighten automated source-fake / neutralized-source gates now that source-composed motifs have stronger render sound. RIOTBOX-1264 remains open through quality-gate, listening-pack, and closeout work.`

## Why This Ticket Existed

MC-202 source-composed motifs projected typed render articulation, but the audio renderer still shaped bass pressure, answer stabs, gate snap, drive, and destructive movement with a generic synth path that did not separate body from bite strongly enough.

## What Shipped

- Added a dedicated MC-202 source phrase sound-design helper on the existing render seam; source-composed plans now shape gain, drive, gate length, envelope curve, sub/saw/pulse/bite mix, transient click, octave drop, destructive pitch dive, and cut timing from typed render-plan articulation values. Primitive/no-source rendering keeps the prior balance. Added audio tests for low-band body vs transient sharpness and no clipping, and documented the RIOTBOX-1276 contract.

## Notes

- The first listening-pack smoke caught a real tonal-case balance regression where primitive/no-source MC-202 pressure masked restore impact. The final implementation limits new production shaping to source-composed render plans, keeping primitive support lanes stable. Listening packs remain human_verdict unverified and quality_proof false.
