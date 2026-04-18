# Benchmarks

Benchmark artifacts live here when Riotbox records a reproducible baseline that should stay visible in the repo.

Use this directory for:

- workflow baselines
- manual or semi-manual operator-path measurements
- benchmark notes that need to stay tied to a commit and fixture

Do not use this directory for:

- raw profiling dumps
- ad hoc scratch notes
- review findings that belong in `docs/reviews/`

Current benchmark artifacts:

- [jam_workflow_baseline_2026-04-17.md](./jam_workflow_baseline_2026-04-17.md)
  First workflow-budget baseline for playable Jam and successful capture on the current example-source path.
- [scene_jump_restore_workflow_baseline_2026-04-18.md](./scene_jump_restore_workflow_baseline_2026-04-18.md)
  First workflow-budget baseline for the current Scene Brain `jump -> restore` recovery loop.
- [scene_timing_readability_baseline_2026-04-18.md](./scene_timing_readability_baseline_2026-04-18.md)
  Current readability baseline for Scene Brain timing, contrast, and recent-result cues across `Jam` and `Log`.
- [scene_guidance_stack_baseline_2026-04-18.md](./scene_guidance_stack_baseline_2026-04-18.md)
  First bounded baseline for the queued-scene guidance stack across `Jam`, `Help`, and `Log`, refreshed for the energy-aware live/restore seam.
