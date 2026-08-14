# `RIOTBOX-1422` P023: Lift the source-backed W-30 resample tap and add a performer-triggered hard variation

- Ticket: `RIOTBOX-1422`
- Title: `P023: Lift the source-backed W-30 resample tap and add a performer-triggered hard variation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1422/p023-lift-the-source-backed-w-30-resample-tap-and-add-a-performer`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Canceled`
- Created: `2026-07-23`
- Started: `2026-07-23`
- Finished: `2026-08-03`
- Branch: `feature/riotbox-1422-p023-lift-the-source-backed-w-30-resample-tap-and-add-a`
- Linear branch: `feature/riotbox-1422-p023-lift-the-source-backed-w-30-resample-tap-and-add-a`
- Assignee: `Owner`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1377 (https://github.com/marang/riotbox/pull/1377), unmerged`
- Merge commit: `None`
- Deleted from Linear: `2026-08-14`
- Verification: `H27 pushed Rust CI passed; dirty H28-H30 manifests preserve exact-path passes, and H30 records local full validation. H27 and H30 structured reviews did not pass; H28-H29 artifact-bound human observations also rejected the intended Hard claim. Closeout manifests are SHA-256-bound and JSON-validated.`
- Docs touched: `docs/benchmarks/w30_resample_h27_development_v1.json; docs/benchmarks/w30_resample_h28_development_v1.json; docs/benchmarks/w30_resample_h29_development_v1.json; docs/benchmarks/w30_resample_h30_development_v1.json; docs/reviews/riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md`
- Follow-ups: `RIOTBOX-1429`, then `RIOTBOX-1428`

## Why This Ticket Existed

Raise the source-backed W-30 resample tap, preserve recognizable source
character, and add a performer-triggered Hard variation with an immediate
audible consequence on the exact live mixer path.

## What Shipped

- Historical H27-H30 manifests and a bounded rejected-experiment closeout.
- No RIOTBOX-1422 recipe or Rust implementation shipped to `main`; PR #1377 remained unmerged.
- No accepted Base implementation shipped because it was not cleanly separable from the abandoned experiment stack.

## Notes

- Linear transitioned to the final `Canceled` disposition on `2026-08-03`
  after this archive reached `main`. The first token-authenticated deletion
  attempt returned `401`; cleanup was retried with a refreshed token on
  `2026-08-14` and completed successfully.
- H27 was technically okay but musically weak and not worth looping.
- H28 sounded duller rather than harder, H29 sounded the same, and H30 offered no useful distinction.
- V7-V10 and the continuous-Base plus sparse parallel-overlay family are retired.
- [Closeout review](../../reviews/riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md)
- Evidence manifests: [H27](../../benchmarks/w30_resample_h27_development_v1.json), [H28](../../benchmarks/w30_resample_h28_development_v1.json), [H29](../../benchmarks/w30_resample_h29_development_v1.json), [H30](../../benchmarks/w30_resample_h30_development_v1.json)
