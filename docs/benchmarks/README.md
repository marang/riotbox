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
- [scene_restore_ready_readability_baseline_2026-04-18.md](./scene_restore_ready_readability_baseline_2026-04-18.md)
  First bounded readability baseline for the explicit `restore <scene>/<energy> ready` cue and help seam on `Jam`.
- [scene_restore_state_contrast_baseline_2026-04-18.md](./scene_restore_state_contrast_baseline_2026-04-18.md)
  First bounded readability baseline for the current `restore waits...` versus `restore ... ready` contrast on `Jam`.
- [scene_post_landed_energy_cue_baseline_2026-04-25.md](./scene_post_landed_energy_cue_baseline_2026-04-25.md)
  Current bounded readability baseline for the post-landed Scene Brain cue using compact `scene/energy` and `909 lift` labels.
- [scene_cue_ladder_baseline_2026-04-25.md](./scene_cue_ladder_baseline_2026-04-25.md)
  First bounded readability baseline for the complete Scene Brain cue ladder across queued, landed, ready, restore, and Log confirmation states.
- [scene_footer_tick_readability_baseline_2026-04-25.md](./scene_footer_tick_readability_baseline_2026-04-25.md)
  First bounded readability baseline for the compact ASCII timing tick in the queued Scene Brain footer cue.
- [scene_contrast_launch_baseline_2026-04-25.md](./scene_contrast_launch_baseline_2026-04-25.md)
  First bounded readability baseline for Scene launch target selection preferring known energy contrast.
- [scene_tr909_support_context_baseline_2026-04-26.md](./scene_tr909_support_context_baseline_2026-04-26.md)
  First bounded readability baseline for Scene-target TR-909 support-context diagnostics.
- [scene_tr909_support_accent_audio_baseline_2026-04-26.md](./scene_tr909_support_accent_audio_baseline_2026-04-26.md)
  First bounded audio-buffer baseline for the Scene-target TR-909 support accent.
- [jam_footer_color_hierarchy_baseline_2026-04-25.md](./jam_footer_color_hierarchy_baseline_2026-04-25.md)
  First bounded readability baseline for the semantic Jam footer color and emphasis hierarchy.
- [capture_do_next_readability_baseline_2026-04-25.md](./capture_do_next_readability_baseline_2026-04-25.md)
  First bounded readability baseline for the Capture `Do Next` hierarchy and audible handoff cues.
- [capture_pending_do_next_readability_baseline_2026-04-25.md](./capture_pending_do_next_readability_baseline_2026-04-25.md)
  Bounded readability baseline for pending-aware Capture `Do Next` states before committed capture fallback guidance.
- [w30_preview_smoke_listening_pack_2026-04-26.md](./w30_preview_smoke_listening_pack_2026-04-26.md)
  Initial local-only W-30 preview listening-pack convention with one deterministic source-window smoke case.
