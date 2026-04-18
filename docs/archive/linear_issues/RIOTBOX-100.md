# `RIOTBOX-100` Measure first playable Jam and first successful capture workflow benchmarks

- Ticket: `RIOTBOX-100`
- Title: `Measure first playable Jam and first successful capture workflow benchmarks`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-100/measure-first-playable-jam-and-first-successful-capture-workflow`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-100-jam-benchmarks`
- Linear branch: `feature/riotbox-100-measure-first-playable-jam-and-first-successful-capture`
- Assignee: `Markus`
- Labels: `None`
- PR: `#93`
- Merge commit: `198f2bd32ffb8cb3a70d9baa3e37fa0e4f4f9b10`
- Deleted from Linear: `Not deleted`
- Verification: `docs-only slice`, branch review + self-review, GitHub Actions `Rust CI` run `#262`
- Docs touched: `docs/benchmarks/README.md`, `docs/benchmarks/jam_workflow_baseline_2026-04-17.md`, `docs/research_decision_log.md`, `docs/README.md`
- Follow-ups: `RIOTBOX-97`, `RIOTBOX-101`, `RIOTBOX-102`

## Why This Ticket Existed

The perform-first Jam shell had already gained a bounded first-run path and clearer recipes, but Riotbox still had no explicit repo-visible benchmark for the two most immediate operator questions: how quickly a fresh ingest session becomes playable and how quickly a user can reach the first successful capture. The roadmap and validation docs already named those workflow goals, so the next honest step was to pin one visible baseline before inventing any new telemetry system.

## What Shipped

- created `docs/benchmarks/` as the repo-visible location for workflow benchmark artifacts
- added the first benchmark baseline for the current example-source Jam path and the shipped `Space -> f -> c` capture loop
- recorded the rationale for an explicit docs-first benchmark method in the research decision log
- updated the docs index so the benchmark artifacts are part of normal repo navigation

## Notes

- this slice intentionally avoided adding runtime instrumentation or analytics; the artifact is a manual benchmark baseline, not a second measurement subsystem
- the baseline captures the current shell reality, including the still-narrow first audible result path, so later UX work can be compared against something explicit
- the next honest follow-up remains the bounded inspect/onramp tightening slices that use this baseline rather than replacing it
