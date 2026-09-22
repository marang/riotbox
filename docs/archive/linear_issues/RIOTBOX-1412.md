# `RIOTBOX-1412` Benchmark W-30 preview snapshot cost and add a revision fast path if justified

- Ticket: `RIOTBOX-1412`
- Title: `Benchmark W-30 preview snapshot cost and add a revision fast path if justified`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1412/benchmark-w-30-preview-snapshot-cost-and-add-a-revision-fast-path-if`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1412-w30-snapshot-cache`
- Linear branch: `feature/riotbox-1412-benchmark-w-30-preview-snapshot-cost-and-add-a-revision-fast`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`, `review-followup`
- PR: `#1527 (https://github.com/marang/riotbox/pull/1527)`
- Merge commit: `c364ea46cf8cab04db46fbdfe083ac21972b7f8b`
- Deleted from Linear: `2026-09-22`
- Verification: `Normal GitHub Rust CI passed; full local source-free CI passed with serial tests, followed by integrated Library tests and strict Clippy.`; `Final same-binary unchanged original/cached 12.731/0.002 us; changed 13.307/11.628 us. Informational local release measurement, not device or xrun qualification.`
- Docs touched: `docs/specs/audio_core_spec.md`, `docs/reviews/riotbox_1412_w30_snapshot_cost_2026-09-22.md`
- Follow-ups: `RIOTBOX-1498 removes the separate Sidecar regression test startup dependency.`

## Why This Ticket Existed

Unchanged W-30 callback snapshots reread and copied 18,432 samples each invocation.

## What Shipped

- Callback-local complete-publication cache skips unchanged sample loads/copies and retains bounded coherent adoption; semantic W-30 snapshot module.

## Notes

- No Session/replay, DSP or musical-policy change. No source/holdout/commercial audio, DAW or listening access. Zero retained solo review findings.
