# `RIOTBOX-1583` Checkpoint integrated callback timing, transport and recording trust boundaries

- Ticket: `RIOTBOX-1583`
- Title: `Checkpoint integrated callback timing, transport and recording trust boundaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1583/checkpoint-integrated-callback-timing-transport-and-recording-trust`
- Project: `P017 | Live Performance Readiness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-16`
- Started: `2026-10-16`
- Finished: `2026-10-16`
- Branch: `feature/riotbox-1583-checkpoint-integrated-callback-timing-transport-and`
- Linear branch: `feature/riotbox-1583-checkpoint-integrated-callback-timing-transport-and`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1688 (https://github.com/marang/riotbox/pull/1688)`
- Merge commit: `a688b5e3f25b6af0f5c02518e59ee0b6105c21e2`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: `26 Core/App, 82 Audio/Sidecar, 12 QA test selections PASS (overlap not unique coverage); independent counterexample rebuild/reruns; full source-free just ci PASS; exact-head native38875077235 both configured jobs PASS; independent report review and short self-review zero additional findings; GitHub comments/reviews empty`
- Docs touched: `docs/reviews/riotbox_1583_current_state_checkpoint_2026-10-16.md`
- Follow-ups: `RIOTBOX-1584 first, then checked ActionId exhaustion; RIOTBOX-1041/1036 remain open. Report source is durable even after Linear deletion.`

## Why This Ticket Existed

Risk-directed active-phase current-state review after generated fixture,
loaded-recorder adapter/observations and observed callback-timing repair began
to interact. Freeze implementation418a6a1d/tree8692f39b, identical to reviewed
1582 candidate720ed9fa. Sample all four Rust crates, Python Sidecar and QA
ownership rather than inferring safety from green CI or counting operational
branches toward five features. Protect accepted P023 Session, timing-fed queue
and recording paths before broader host/endurance or DAW qualification.

## What Shipped

- A bounded system/contract/evidence report with two reproduced P2 findings:
  control→callback revision guards with Relaxed payload can accept mixed
  generations; saturated ActionId allocation can save duplicate history that
  normal restore and replay subsequently reject.
- Original control proof is a byte-identical extract-based Loom model, not
  actual hardware/runtime proof. Coordinator independently checked the source
  ranges/hash, rebuilt and reproduced `(false,132,8)` between two complete
  states. The MAX/MAX−1 proof uses real public Core/App APIs and three ordinary
  toggles across two small source/graph/capture-free Sessions; coordinator rebuilt
  and repeated it. These negative repros are not passing integrity tests.
- Existing generated controls pass but do not cover these new counterexamples.
  No other P0–P3 defect survived the sampled review. Independent report review
  verified provenance/counts/limits; no product repair is bundled in the report.

## Notes

- The control defect is pre-existing and distinct from1582's SC callback→App
  observed timing owner. Immediate repair1584 subsequently reached an
  actual-source Loom RED before implementation paused for the user's explicit
  Done-ticket cleanup request. Its five uncommitted Rust paths stay preserved in
  the original worktree; the separate archive worktree changes no product code.
- Linear's free limit blocked both the second defect's issue and a separate
  cleanup issue. The user explicitly authorized archiving/deleting all Done
  tickets; normal closeout of this existing issue therefore owns the bounded
  archive-only followup branch `feature/riotbox-1583-done-closeout`. The report
  was actually merged and marked Done before this record was prepared; the
  issue was reopened solely to keep that authorized closeout Linear-first.
- Initial inventory:33 unarchived Done records (1501 and1551–1582), plus63
  already archived records whose exact committed per-ticket archives were
  verified. Include this just-completed report as the34th new archive. Delete
  only after durable archive merge and fresh Done/identity verification; never
  touch open tickets or infer deletion from a planned timestamp. The separately
  approved archived1550 deletion succeeded; its old archive remains intact.
- Full-runtime memory models, aggregate/RSS memory, concurrent-writer/power-loss
  filesystem guarantees, unsampled DSP/UI, physical-device/endurance, DAW import
  and human taste remain outside the report. Original ALSA/backend cause stays
  unknown. No new host/source/audio/ignored-attempt access, musical/hardness,
  source-general, latency, endurance or release qualification follows.
- Specifically approved late Jev advisory used historical pre-checkpoint status
  (1054 input/156 output tokens), favoring lifecycle inspection. It is not defect
  evidence, host authorization or a current test result.
