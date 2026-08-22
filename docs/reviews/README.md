# Riotbox Review Artifacts

Review documents in this directory are point-in-time evidence. They preserve what
was true, risky, or recommended at the moment of a review. They are not the live
backlog and they should not be treated as proof that a finding is still open.

Before turning a review finding or recommended follow-up into a new ticket:

1. check the current implementation on `main`
2. search current Linear issues, including recently done or canceled issues
3. search `docs/archive/linear_issues/` only when ticket history is needed
4. decide whether the finding is still open, already shipped, duplicate,
   superseded, or intentionally deferred

If the finding is already closed, do not create a new implementation ticket.
Reference the shipped ticket or archive entry instead. If the finding is still
open, create the smallest bounded ticket that fits the current roadmap phase.

When a newer review refreshes an older one, link the newer document from the old
review when doing so is useful, but do not rewrite historical findings just to
make old reviews look current.

External reviews follow the same freshness rule. Treat them as useful
point-in-time evidence, not as automatically current backlog. If an external
review cites source-level risks that are partly stale, capture a refresh note
that separates the still-valid engineering risk from the already-shipped or
superseded wording before creating Linear tickets.

Current module-ownership refreshes:

- `external_review_refresh_2026-05-22.md`: external-review freshness check for
  `jam_app`, audio QA, and runtime ownership findings.
- `tui_include_shell_audit_2026-05-22.md`: TUI include-shell audit and
  leaf-first module-conversion recommendation.

Current P012 source-timing refreshes:

- `p012_real_source_timing_confidence_review_2026-05-22.md`: current local
  example confidence rows and the next downbeat-ambiguity surface slice.

Current P023 rejected-experiment closeouts:

- [riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md](./riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md):
  artifact-bound H27-H30 verdicts, retired V7-V10 mechanism family, and the
  narrow re-extraction boundary for stack-only work that never reached `main`.
- [riotbox_1436_tr909_impact_pocket_development_2026-08-14.md](./riotbox_1436_tr909_impact_pocket_development_2026-08-14.md):
  technically clean but inaudible local-impact result and removed mechanism.
- [riotbox_1437_tr909_counter_rhythm_development_2026-08-14.md](./riotbox_1437_tr909_counter_rhythm_development_2026-08-14.md):
  dense success, source-diversity failure, and removed counter-rhythm family.
- [riotbox_1438_cut_hit_return_development_2026-08-15.md](./riotbox_1438_cut_hit_return_development_2026-08-15.md):
  useful underlying transformations, near-identical full-arc verdict, and
  removed one-key composition.
- [riotbox_1417_mc202_realized_role_development_rejection_2026-08-16.md](./riotbox_1417_mc202_realized_role_development_rejection_2026-08-16.md):
  three bounded MC-202 instigator variants, no provisional human keep, and the
  retired retrigger/full-bed-cut mechanism family.
- [riotbox_1449_mc202_answer_development_rejection_2026-08-21.md](./riotbox_1449_mc202_answer_development_rejection_2026-08-21.md):
  source-derived sparse-answer selection, technically valid but perceptually
  unchanged full-mix A/B, and the separate intent-role fail-closed follow-up.

Current P023 algorithm-value refresh:

- [riotbox_1441_w30_hook_turnaround_transfer_observation_2026-08-18.md](./riotbox_1441_w30_hook_turnaround_transfer_observation_2026-08-18.md):
  five-source listening observations, source-dependent usefulness, improved
  phrase-scale review method, and formal fail-closed qualification boundary.
- [riotbox_1440_w30_hook_turnaround_development_2026-08-16.md](./riotbox_1440_w30_hook_turnaround_development_2026-08-16.md):
  passed W-30 hook-turnaround exploration, three-source product qualification,
  and formal human review.
- [riotbox_1442_w30_pitch_dive_development_2026-08-18.md](./riotbox_1442_w30_pitch_dive_development_2026-08-18.md):
  provisionally kept a source-recognizable continuous W-30 pitch-dive exit and
  froze the exact recipe before transfer observation and product integration.
- [riotbox_1443_w30_pitch_dive_transfer_observation_2026-08-18.md](./riotbox_1443_w30_pitch_dive_transfer_observation_2026-08-18.md):
  positive four-source transfer observation for the unchanged W-30 Pitch Dive
  across dense-break, sparse-drum, and tonal-riff material.
- [riotbox_1444_w30_pitch_dive_product_qualification_2026-08-19.md](./riotbox_1444_w30_pitch_dive_product_qualification_2026-08-19.md):
  source-blind product integration, passed four-source exact-RuntimeMix matrix,
  and positive formal human product review for the frozen W-30 Pitch Dive.
- [riotbox_1454_tonal_live_journey_v1_rejection_2026-08-22.md](./riotbox_1454_tonal_live_journey_v1_rejection_2026-08-22.md):
  exact-live rejection of the weak tonal TR-909 support pulse and overlong
  Pitch Dive presentation before the versioned live-policy correction.
- [riotbox_1454_tonal_live_journey_v2_acceptance_2026-08-22.md](./riotbox_1454_tonal_live_journey_v2_acceptance_2026-08-22.md):
  W-30-only tonal capture, Pitch Dive, ordinary re-entry, and restart/recall
  journey with a positive structured human review.
- [riotbox_1445_w30_filter_slam_development_2026-08-19.md](./riotbox_1445_w30_filter_slam_development_2026-08-19.md):
  provisionally kept the exact eight-beat W-30 Filter Slam after a shorter
  audible but underdeveloped variant, with product integration still separate.
- [riotbox_1446_w30_filter_slam_product_qualification_2026-08-19.md](./riotbox_1446_w30_filter_slam_product_qualification_2026-08-19.md):
  source-blind product integration, passed four-source exact-RuntimeMix matrix,
  and positive formal human product review for the frozen W-30 Filter Slam.

- [riotbox_1439_delivery_system_audit_2026-08-16.md](./riotbox_1439_delivery_system_audit_2026-08-16.md):
  environment, research, roadmap, plans, workflow, audio-QA, and skill audit;
  accepted discovery-before-qualification correction and direct RIOTBOX-1417
  handoff.
- [riotbox_1433_audible_algorithm_value_audit_2026-08-12.md](./riotbox_1433_audible_algorithm_value_audit_2026-08-12.md):
  portfolio-wide retain/replace/retire evidence and the frozen bounded handoff
  to RIOTBOX-1432's product-owned W-30 hook selection.
- [riotbox_1432_w30_source_hook_selection_2026-08-14.md](./riotbox_1432_w30_source_hook_selection_2026-08-14.md):
  frozen three-source comparison, exact RuntimeMix identities, and fail-closed
  no-winner verdict with current product behavior retained.
