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

- [audio_qa_artifact_convention_2026-04-26.md](./audio_qa_artifact_convention_2026-04-26.md)
  Initial local-only baseline-vs-candidate artifact convention for generated audio QA outputs.
- [audio_qa_listening_review_template_2026-04-26.md](./audio_qa_listening_review_template_2026-04-26.md)
  Local human listening-review template for `notes.md` files beside generated audio QA artifacts.
- [observer_audio_correlation_template_2026-04-29.md](./observer_audio_correlation_template_2026-04-29.md)
  Local template for correlating `riotbox-app --observer` control-path evidence with generated audio QA `manifest.json` output evidence, with an optional generated Markdown summary helper.
- [observer_audio_summary_json_contract_2026-04-29.md](./observer_audio_summary_json_contract_2026-04-29.md)
  Schema marker and compatibility contract for `observer_audio_correlate --json` summaries.
- [listening_manifest_schema_policy_2026-04-29.md](./listening_manifest_schema_policy_2026-04-29.md)
  Schema version and compatibility policy for local audio QA `manifest.json` files.
- [listening_manifest_v1_json_contract_2026-04-29.md](./listening_manifest_v1_json_contract_2026-04-29.md)
  Field-level JSON contract for local audio QA listening manifest schema version 1.
- [p011_replay_family_manifest.json](./p011_replay_family_manifest.json)
  Machine-checkable P011 supported replay-family evidence index.
- [p011_exit_evidence_manifest.json](./p011_exit_evidence_manifest.json)
  Machine-checkable P011 exit evidence index across replay, recovery, export, and stage-style stability categories. `just p011-exit-evidence-gate` executes all bounded categories from this index with global command deduplication; the per-category `p011-*-evidence-gate` recipes remain available for targeted local checks.
- [representative_source_showcase_2026-05-07.md](./representative_source_showcase_2026-05-07.md)
  Local multi-source Feral grid showcase generator for source-response review after the P011 diversity fixes.
- [melodic_source_chop_showcase_2026-05-20.md](./melodic_source_chop_showcase_2026-05-20.md)
  Local non-drum source-chop showcase that preserves unavailable Source Timing as a boundary instead of treating melodic sources as Feral-grid drum-support examples.
- [product_export_reproducibility_boundary_2026-05-07.md](./product_export_reproducibility_boundary_2026-05-07.md)
  Normalized product-export reproducibility boundary for the current Feral grid generated-support full-mix seam.
- [stage_style_stability_proof_2026-05-07.md](./stage_style_stability_proof_2026-05-07.md)
  Normalized repeated-run proof for the generated stage-style restore-diversity observer/audio stability seam.
- [source_timing_fixture_seeds_2026-05-07.md](./source_timing_fixture_seeds_2026-05-07.md)
  Initial P012 source-timing fixture seed catalog and validation gate for BPM, grid, confidence, ambiguity, and degraded-policy expectations.
- [source_timing_analyzer_skeleton_2026-05-07.md](./source_timing_analyzer_skeleton_2026-05-07.md)
  Initial P012 Rust timing analyzer skeleton that maps fixture seeds into SourceGraph `TimingModel` payloads.
- [source_timing_fixture_evaluator_2026-05-07.md](./source_timing_fixture_evaluator_2026-05-07.md)
  Initial P012 evaluator that compares analyzer `TimingModel` output against fixture timing expectations.
- [source_timing_wav_probe_2026-05-07.md](./source_timing_wav_probe_2026-05-07.md)
  Initial P012 WAV onset-envelope probe for deterministic source timing input features.
- [source_timing_bpm_candidates_2026-05-07.md](./source_timing_bpm_candidates_2026-05-07.md)
  Initial P012 synthetic WAV/probe BPM candidate spike with preserved half/double-time alternatives.
- [source_timing_downbeat_ambiguity_2026-05-07.md](./source_timing_downbeat_ambiguity_2026-05-07.md)
  Initial P012 downbeat phase ambiguity scaffold for probe BPM candidates.
- [source_timing_candidate_confidence_report_2026-05-07.md](./source_timing_candidate_confidence_report_2026-05-07.md)
  Initial P012 candidate confidence report seam for probe timing QA.
- [source_timing_example_readiness_2026-05-07.md](./source_timing_example_readiness_2026-05-07.md)
  Local real-example readiness check for Feral grid packs, including when current example sources still need explicit BPM instead of auto timing.
- [source_timing_generated_probe_matrix_2026-05-08.md](./source_timing_generated_probe_matrix_2026-05-08.md)
  Generated Source Timing probe matrix covering strong grid-lock, silence degradation, and flat-pulse beat-without-downbeat ambiguity on the real CLI path.
- [source_timing_example_probe_report_2026-05-11.md](./source_timing_example_probe_report_2026-05-11.md)
  Optional local real-example Source Timing report baseline for the documented Beat/DH example WAVs.
- [source_timing_example_probe_report_2026-05-21.md](./source_timing_example_probe_report_2026-05-21.md)
  Refreshed local real-example Source Timing report baseline after Beat20 moved from generic weak timing to reviewable downbeat ambiguity.
- [beat03_auto_feral_grid_source_timing_2026-05-21.md](./beat03_auto_feral_grid_source_timing_2026-05-21.md)
  Local Beat03 Feral-grid auto-BPM proof showing Source Timing drives the grid with visible manual-confirm evidence.
- [beat08_auto_feral_grid_source_timing_2026-05-21.md](./beat08_auto_feral_grid_source_timing_2026-05-21.md)
  Local Beat08 Feral-grid auto-BPM proof for the primary recipe source.
- [beat20_auto_feral_grid_fallback_2026-05-21.md](./beat20_auto_feral_grid_fallback_2026-05-21.md)
  Local Beat20 Feral-grid auto-BPM fallback proof for ambiguous downbeat evidence.
- [dh_beatc_auto_feral_grid_source_timing_2026-05-21.md](./dh_beatc_auto_feral_grid_source_timing_2026-05-21.md)
  Local DH_BeatC Feral-grid auto-BPM proof showing the same cautious Source Timing grid path.
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
- [lane_recipe_listening_pack_2026-04-26.md](./lane_recipe_listening_pack_2026-04-26.md)
  Initial local lane-level listening-pack harness for documented Jam recipes outside the W-30 preview path.
- `scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json`
  Machine-checkable 10/10 sound-product readiness rubric for technical pass,
  diagnostic evidence, automated promising, human weak/pass, demo-ready, and
  release-ready states.
- `scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json`
  Machine-checkable release-grade musician demo-bank contract that separates
  pass, weak, failed, and `human_verdict: unverified` examples while requiring
  dense-break and non-dense source-family coverage.
- [professional_output_suite_v1_2026-06-04.md](./professional_output_suite_v1_2026-06-04.md)
  Aggregate professional-output suite report that hashes and summarizes dense-break, source-matrix, source-WAV, listening-pack, and destructive-variation gates without claiming a human musical pass.
- [sound_excellence_source_corpus_v1_2026-06-05.md](./sound_excellence_source_corpus_v1_2026-06-05.md)
  P023 real-source coverage contract for dense breaks, sparse drums, tonal riffs, pad/noise, weak sources, and bad-timing policy material.
- [weak_output_fix_routing_v1_2026-06-05.md](./weak_output_fix_routing_v1_2026-06-05.md)
  P023 weak-output actionability diagnostic that maps failure codes and listening-review tags to concrete production fix categories without claiming musical pass.
- [jam_footer_color_hierarchy_baseline_2026-04-25.md](./jam_footer_color_hierarchy_baseline_2026-04-25.md)
  First bounded readability baseline for the semantic Jam footer color and emphasis hierarchy.
- [capture_do_next_readability_baseline_2026-04-25.md](./capture_do_next_readability_baseline_2026-04-25.md)
  First bounded readability baseline for the Capture `Do Next` hierarchy and audible handoff cues.
- [capture_pending_do_next_readability_baseline_2026-04-25.md](./capture_pending_do_next_readability_baseline_2026-04-25.md)
  Bounded readability baseline for pending-aware Capture `Do Next` states before committed capture fallback guidance.
- [w30_preview_smoke_listening_pack_2026-04-26.md](./w30_preview_smoke_listening_pack_2026-04-26.md)
  Initial local-only W-30 preview listening-pack convention with one deterministic source-window smoke case.
