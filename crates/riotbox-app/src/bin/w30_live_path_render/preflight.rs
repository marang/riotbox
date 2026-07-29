use std::{fs, path::Path};

use riotbox_app::jam_app::EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE;
use riotbox_audio::{
    source_audio::SourceAudioCache,
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
    w30::{
        W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY,
        W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE, W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE,
        W30ResampleLowImpactDecision, W30ResampleTapState,
    },
};
use riotbox_core::{
    source_graph::{
        MeterHint, SourceTimingProbeBpmCandidatePolicy, SourceTimingProbeReadinessStatus,
        TempoGuidedTimingDecision, TempoGuidedTimingEvidence, TimingHypothesisKind, TimingModel,
        source_timing_grid_use, source_timing_grid_use_from_timing_model,
        source_timing_probe_readiness_report, tempo_guided_timing_hypothesis,
    },
    w30::{W30HardIntent, W30HardIntentOutcome},
};
use serde::Serialize;

const PREFLIGHT_REPORT_FILE: &str = "w30-reachability-preflight.json";

#[derive(Clone, Debug, Serialize)]
pub(super) struct W30ReachabilityPreflightReport {
    schema: &'static str,
    schema_version: u32,
    source_path: String,
    timing: W30TimingReachability,
    projection: Option<W30HardProjectionReachability>,
    candidate_wav_generation_eligible_after_preflight: bool,
    blockers: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct W30TimingReachability {
    requested_bpm: f32,
    requested_downbeat_seconds: Option<f32>,
    primary_bpm: Option<f32>,
    bpm_delta: Option<f32>,
    bpm_match_tolerance: f32,
    readiness: &'static str,
    requires_manual_confirm: bool,
    grid_use: &'static str,
    confirmation_route: &'static str,
    tempo_guided_evidence: Option<TempoGuidedTimingEvidence>,
    product_projection_allowed: bool,
    product_graph_primary_bpm: Option<f32>,
    product_graph_grid_use: Option<&'static str>,
    product_graph_matches_confirmation_route: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
struct W30HardProjectionReachability {
    evaluated_through_product_queue_commit_projection: bool,
    requested_intent: &'static str,
    intent_outcome: &'static str,
    hard_policy: &'static str,
    hard_suitability: &'static str,
    low_impact_recipe: &'static str,
    low_impact_role: &'static str,
    low_impact_decision: &'static str,
    low_impact_candidate_count: u8,
    low_impact_selected_slot: u8,
    low_impact_selected_onset_cursor: u16,
    low_impact_attack_window_proxy_frames: u16,
    low_impact_body_window_proxy_frames: u16,
    low_band_attack_share: f32,
    low_band_attack_share_min: f32,
    low_band_attack_over_body: f32,
    low_band_attack_over_body_min: f32,
    low_band_attack_over_source: f32,
    low_band_attack_over_source_min: f32,
    exact_callback_calibration_applicable: bool,
    exact_callback_evaluated: bool,
    exact_callback_calibrated: bool,
    predicted_level_ratio: f32,
    predicted_presence_head_ratio: f32,
    predicted_body_ratio: f32,
    predicted_crest_ratio: f32,
    presence_head_wet: f32,
    impact_body_eq_gain_db: f32,
    impact_presence_gain: f32,
    candidate_requirement_satisfied: bool,
}

impl W30ReachabilityPreflightReport {
    pub(super) fn from_timing(source_path: &Path, timing: W30TimingReachability) -> Self {
        let blockers = if timing.product_projection_allowed {
            Vec::new()
        } else {
            vec![timing_blocker(timing.confirmation_route)]
        };
        Self {
            schema: "riotbox.w30_reachability_preflight.v1",
            schema_version: 1,
            source_path: source_path.display().to_string(),
            timing,
            projection: None,
            candidate_wav_generation_eligible_after_preflight: false,
            blockers,
        }
    }

    pub(super) fn timing_allows_product_projection(&self) -> bool {
        self.timing.product_projection_allowed
    }

    pub(super) fn record_product_timing(&mut self, timing: &TimingModel) {
        let product_primary = timing.primary_hypothesis();
        let product_primary_bpm = product_primary.map(|hypothesis| hypothesis.bpm);
        let matches = match self.timing.confirmation_route {
            "musician_manual_bpm_and_downbeat" => product_primary.is_some_and(|hypothesis| {
                hypothesis.kind == TimingHypothesisKind::Manual
                    && (hypothesis.bpm - self.timing.requested_bpm).abs() <= 0.001
            }),
            "source_derived_downbeat_at_external_tempo" => {
                let selected_downbeat = self
                    .timing
                    .tempo_guided_evidence
                    .as_ref()
                    .and_then(|evidence| evidence.selected_downbeat_seconds);
                product_primary.is_some_and(|hypothesis| {
                    hypothesis.kind == TimingHypothesisKind::TempoGuided
                        && (hypothesis.bpm - self.timing.requested_bpm).abs() <= 0.001
                        && selected_downbeat.is_some_and(|selected| {
                            hypothesis
                                .bar_grid
                                .first()
                                .is_some_and(|bar| (bar.start_seconds - selected).abs() <= 0.001)
                        })
                })
            }
            _ => match (self.timing.primary_bpm, product_primary_bpm) {
                (Some(preflight), Some(product)) => (preflight - product).abs() <= 0.001,
                (None, None) => true,
                _ => false,
            },
        };
        self.timing.product_graph_primary_bpm = product_primary_bpm;
        self.timing.product_graph_grid_use =
            Some(source_timing_grid_use_from_timing_model(timing).label());
        self.timing.product_graph_matches_confirmation_route = Some(matches);
        if !matches {
            self.timing.product_projection_allowed = false;
            self.blockers
                .push("product_timing_did_not_match_confirmation_route");
        }
    }

    pub(super) fn record_projection(&mut self, state: &W30ResampleTapState) {
        let performer_impact_committed = state.hard_intent == Some(W30HardIntent::Impact);
        let performer_impact_realized =
            state.hard_intent_outcome == W30HardIntentOutcome::RealizedImpact;
        let applicable = state.exact_hit_shaper_calibration_applicable();
        let calibration_was_applicable = applicable
            || (state.hard_calibration.exact_callback_evaluated
                && state.hard_low_impact.decision
                    == W30ResampleLowImpactDecision::ExactCallbackRejected);
        let requirement_satisfied = performer_impact_committed
            && performer_impact_realized
            && applicable
            && state.hard_calibration.exact_callback_evaluated
            && state.hard_calibration.exact_callback_calibrated;
        self.projection = Some(W30HardProjectionReachability {
            evaluated_through_product_queue_commit_projection: true,
            requested_intent: state.hard_intent.map_or("none", W30HardIntent::label),
            intent_outcome: state.hard_intent_outcome.label(),
            hard_policy: state.hard_policy.label(),
            hard_suitability: state.hard_suitability.status.label(),
            low_impact_recipe: state.hard_low_impact.recipe.label(),
            low_impact_role: state.hard_low_impact.role.label(),
            low_impact_decision: state.hard_low_impact.decision.label(),
            low_impact_candidate_count: state.hard_low_impact.candidate_count,
            low_impact_selected_slot: state.hard_low_impact.selected_slot,
            low_impact_selected_onset_cursor: state.hard_low_impact.selected_onset_cursor,
            low_impact_attack_window_proxy_frames: state.hard_low_impact.attack_window_proxy_frames,
            low_impact_body_window_proxy_frames: state.hard_low_impact.body_window_proxy_frames,
            low_band_attack_share: state.hard_low_impact.low_band_attack_share,
            low_band_attack_share_min: W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE,
            low_band_attack_over_body: state.hard_low_impact.low_band_attack_over_body,
            low_band_attack_over_body_min: W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY,
            low_band_attack_over_source: state.hard_low_impact.low_band_attack_over_source,
            low_band_attack_over_source_min: W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE,
            exact_callback_calibration_applicable: calibration_was_applicable,
            exact_callback_evaluated: state.hard_calibration.exact_callback_evaluated,
            exact_callback_calibrated: state.hard_calibration.exact_callback_calibrated,
            predicted_level_ratio: state.hard_calibration.predicted_compensated_level_ratio,
            predicted_presence_head_ratio: state.hard_calibration.predicted_presence_head_ratio,
            predicted_body_ratio: state.hard_calibration.predicted_level_matched_body_ratio,
            predicted_crest_ratio: state.hard_calibration.predicted_crest_ratio,
            presence_head_wet: state.hard_low_impact.presence_head_wet,
            impact_body_eq_gain_db: state.hard_calibration.impact_body_eq_gain_db,
            impact_presence_gain: state
                .hard_low_impact
                .recipe
                .calibrated_presence_gain(state.hard_calibration.impact_body_eq_gain_db),
            candidate_requirement_satisfied: requirement_satisfied,
        });
        self.candidate_wav_generation_eligible_after_preflight =
            self.timing.product_projection_allowed
                && self.timing.product_graph_matches_confirmation_route == Some(true)
                && requirement_satisfied;
        if !requirement_satisfied {
            self.blockers.push(
                if state.hard_low_impact.decision
                    == W30ResampleLowImpactDecision::ExactCallbackRejected
                {
                    "exact_hit_shaper_calibration_rejected"
                } else if !performer_impact_committed {
                    "performer_impact_intent_not_committed"
                } else if !performer_impact_realized {
                    "performer_impact_intent_not_realized"
                } else if !applicable {
                    "exact_hit_shaper_calibration_not_applicable"
                } else if !state.hard_calibration.exact_callback_evaluated {
                    "exact_hit_shaper_calibration_not_evaluated"
                } else {
                    "exact_hit_shaper_calibration_rejected"
                },
            );
        }
    }

    pub(super) fn candidate_wav_generation_eligible_after_preflight(&self) -> bool {
        self.candidate_wav_generation_eligible_after_preflight
    }
}

fn timing_blocker(route: &str) -> &'static str {
    match route {
        "explicit_bpm_mismatch" => "explicit_bpm_mismatch",
        "primary_grid_unavailable" => "primary_grid_unavailable",
        "invalid_explicit_bpm" => "invalid_explicit_bpm",
        "invalid_explicit_downbeat" => "invalid_explicit_downbeat",
        _ => "source_timing_not_reachable",
    }
}

pub(super) fn analyze_timing_reachability(
    source_path: &Path,
    requested_bpm: f32,
    requested_downbeat_seconds: Option<f32>,
) -> Result<W30TimingReachability, Box<dyn std::error::Error>> {
    let source = SourceAudioCache::load_pcm_wav(source_path)?;
    let probe = analyze_source_timing_probe(&source, SourceTimingProbeConfig::default());
    let input = probe.bpm_candidate_input(
        source_path.display().to_string(),
        MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    );
    let readiness = source_timing_probe_readiness_report(
        &input,
        SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness(),
    );
    let tempo_guided_needed = requested_downbeat_seconds.is_none()
        && requested_bpm.is_finite()
        && requested_bpm > 0.0
        && readiness.primary_bpm.is_none_or(|primary| {
            (requested_bpm - primary).abs() > EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE
        });
    let tempo_guided =
        tempo_guided_needed.then(|| tempo_guided_timing_hypothesis(&input, requested_bpm).evidence);
    Ok(classify_timing_reachability_with_tempo_guided(
        requested_bpm,
        requested_downbeat_seconds,
        readiness.primary_bpm,
        readiness.readiness,
        readiness.requires_manual_confirm,
        source_timing_grid_use(&readiness).label(),
        tempo_guided,
    ))
}

#[cfg(test)]
fn classify_timing_reachability(
    requested_bpm: f32,
    requested_downbeat_seconds: Option<f32>,
    primary_bpm: Option<f32>,
    readiness: SourceTimingProbeReadinessStatus,
    requires_manual_confirm: bool,
    grid_use: &'static str,
) -> W30TimingReachability {
    classify_timing_reachability_with_tempo_guided(
        requested_bpm,
        requested_downbeat_seconds,
        primary_bpm,
        readiness,
        requires_manual_confirm,
        grid_use,
        None,
    )
}

fn classify_timing_reachability_with_tempo_guided(
    requested_bpm: f32,
    requested_downbeat_seconds: Option<f32>,
    primary_bpm: Option<f32>,
    readiness: SourceTimingProbeReadinessStatus,
    requires_manual_confirm: bool,
    grid_use: &'static str,
    tempo_guided_evidence: Option<TempoGuidedTimingEvidence>,
) -> W30TimingReachability {
    let bpm_delta = primary_bpm.map(|primary| (requested_bpm - primary).abs());
    let (confirmation_route, product_projection_allowed) =
        if !requested_bpm.is_finite() || requested_bpm <= 0.0 {
            ("invalid_explicit_bpm", false)
        } else if requested_downbeat_seconds
            .is_some_and(|downbeat| !downbeat.is_finite() || downbeat < 0.0)
        {
            ("invalid_explicit_downbeat", false)
        } else if requested_downbeat_seconds.is_some() {
            ("musician_manual_bpm_and_downbeat", true)
        } else if primary_bpm.is_some()
            && bpm_delta.is_some_and(|delta| delta <= EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE)
        {
            ("explicit_bpm_matches_rust_primary", true)
        } else if tempo_guided_evidence
            .as_ref()
            .is_some_and(|evidence| evidence.decision == TempoGuidedTimingDecision::Selected)
        {
            ("source_derived_downbeat_at_external_tempo", true)
        } else if primary_bpm.is_none() {
            ("primary_grid_unavailable", false)
        } else if bpm_delta.is_some_and(|delta| delta > EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE) {
            ("explicit_bpm_mismatch", false)
        } else {
            ("explicit_bpm_matches_rust_primary", true)
        };
    W30TimingReachability {
        requested_bpm,
        requested_downbeat_seconds,
        primary_bpm,
        bpm_delta,
        bpm_match_tolerance: EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE,
        readiness: timing_readiness_label(readiness),
        requires_manual_confirm,
        grid_use,
        confirmation_route,
        tempo_guided_evidence,
        product_projection_allowed,
        product_graph_primary_bpm: None,
        product_graph_grid_use: None,
        product_graph_matches_confirmation_route: None,
    }
}

fn timing_readiness_label(readiness: SourceTimingProbeReadinessStatus) -> &'static str {
    match readiness {
        SourceTimingProbeReadinessStatus::Unavailable => "unavailable",
        SourceTimingProbeReadinessStatus::Weak => "weak",
        SourceTimingProbeReadinessStatus::NeedsReview => "needs_review",
        SourceTimingProbeReadinessStatus::Ready => "ready",
    }
}

pub(super) fn write_preflight_report(
    output_dir: &Path,
    report: &W30ReachabilityPreflightReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_vec_pretty(report)?;
    fs::write(output_dir.join(PREFLIGHT_REPORT_FILE), &json)?;
    println!("{}", String::from_utf8(json)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use riotbox_audio::w30::{
        W30_RESAMPLE_SOURCE_WINDOW_LEN, W30ResampleLowImpactDecision, W30ResampleLowImpactRecipe,
        W30ResampleLowImpactRole, W30ResampleSourceWindow, W30ResampleTapHardPolicy,
        W30ResampleTapState, W30ResampleTapVariation,
    };
    use riotbox_core::source_graph::{TimingDegradedPolicy, TimingHypothesis, TimingQuality};

    use super::*;

    fn timing_report(primary_bpm: Option<f32>, requested_bpm: f32) -> W30TimingReachability {
        classify_timing_reachability(
            requested_bpm,
            None,
            primary_bpm,
            SourceTimingProbeReadinessStatus::Weak,
            true,
            "manual_confirm_only",
        )
    }

    fn applicable_hit_shaper_state() -> W30ResampleTapState {
        let mut state = W30ResampleTapState {
            variation: W30ResampleTapVariation::HardDamage,
            hard_intent: Some(W30HardIntent::Impact),
            hard_intent_outcome: W30HardIntentOutcome::RealizedImpact,
            hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
            tempo_bpm: 130.0,
            source_audio: Some(Box::new(W30ResampleSourceWindow {
                source_revision: 1,
                source_start_frame: 0,
                source_sample_rate: 48_000,
                source_frame_count: 1,
                sample_count: 1,
                samples: [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN],
            })),
            ..Default::default()
        };
        state.hard_low_impact.recipe = W30ResampleLowImpactRecipe::SourceImpactShaperV4;
        state.hard_low_impact.role = W30ResampleLowImpactRole::TransientImpact;
        state.hard_low_impact.decision = W30ResampleLowImpactDecision::SourceHitSelected;
        state.hard_low_impact.candidate_count = 5;
        state.hard_low_impact.selected_slot = 3;
        state.hard_low_impact.selected_onset_cursor = 1_234;
        state.hard_low_impact.attack_window_proxy_frames = 48;
        state.hard_low_impact.body_window_proxy_frames = 96;
        state.hard_low_impact.low_band_attack_share = 0.24;
        state.hard_low_impact.low_band_attack_over_body = 1.65;
        state.hard_low_impact.low_band_attack_over_source = 0.42;
        state.hard_calibration.exact_callback_evaluated = true;
        state.hard_calibration.exact_callback_calibrated = true;
        state
    }

    fn primary_timing(kind: TimingHypothesisKind, bpm: f32) -> TimingModel {
        let hypothesis_id = "test-primary".to_string();
        TimingModel {
            bpm_estimate: Some(bpm),
            bpm_confidence: 1.0,
            meter_hint: Some(MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            }),
            hypotheses: vec![TimingHypothesis {
                hypothesis_id: hypothesis_id.clone(),
                kind,
                bpm,
                meter: MeterHint {
                    beats_per_bar: 4,
                    beat_unit: 4,
                },
                confidence: 1.0,
                score: 1.0,
                beat_grid: Vec::new(),
                bar_grid: Vec::new(),
                phrase_grid: Vec::new(),
                anchors: Vec::new(),
                drift: Vec::new(),
                groove: Vec::new(),
                quality: TimingQuality::Low,
                warnings: Vec::new(),
                provenance: Vec::new(),
            }],
            primary_hypothesis_id: Some(hypothesis_id),
            quality: TimingQuality::Low,
            degraded_policy: TimingDegradedPolicy::ManualConfirm,
            ..Default::default()
        }
    }

    #[test]
    fn timing_preflight_rejects_h15_and_h16_alias_mismatches() {
        let h15 = timing_report(Some(141.509_45), 190.0);
        assert_eq!(h15.confirmation_route, "explicit_bpm_mismatch");
        assert!(!h15.product_projection_allowed);

        let h16 = timing_report(Some(154.639_18), 140.0);
        assert_eq!(h16.confirmation_route, "explicit_bpm_mismatch");
        assert!(!h16.product_projection_allowed);
    }

    #[test]
    fn timing_preflight_rejects_unavailable_one_shot_grid() {
        let one_shot = timing_report(None, 130.0);
        assert_eq!(one_shot.confirmation_route, "primary_grid_unavailable");
        assert!(!one_shot.product_projection_allowed);
    }

    #[test]
    fn timing_preflight_rejects_invented_or_invalid_downbeat() {
        let invalid = classify_timing_reachability(
            130.0,
            Some(f32::NAN),
            Some(130.1),
            SourceTimingProbeReadinessStatus::NeedsReview,
            true,
            "manual_confirm_only",
        );
        assert_eq!(invalid.confirmation_route, "invalid_explicit_downbeat");
        assert!(!invalid.product_projection_allowed);
    }

    #[test]
    fn independently_confirmed_bpm_and_downbeat_can_replace_analyzer_alias() {
        let manual = classify_timing_reachability(
            190.0,
            Some(0.25),
            Some(141.509_45),
            SourceTimingProbeReadinessStatus::Weak,
            true,
            "manual_confirm_only",
        );
        assert_eq!(
            manual.confirmation_route,
            "musician_manual_bpm_and_downbeat"
        );
        assert!(manual.product_projection_allowed);
        assert!(manual.bpm_delta.expect("delta") > EXPLICIT_SOURCE_BPM_MATCH_TOLERANCE);

        let mut report =
            W30ReachabilityPreflightReport::from_timing(Path::new("manual.wav"), manual);
        report.record_product_timing(&primary_timing(TimingHypothesisKind::Manual, 190.0));
        assert!(report.timing.product_projection_allowed);
        assert_eq!(
            report.timing.product_graph_matches_confirmation_route,
            Some(true)
        );
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn source_derived_phase_can_replace_analyzer_alias_without_becoming_manual_truth() {
        let evidence = TempoGuidedTimingEvidence {
            decision: TempoGuidedTimingDecision::Selected,
            requested_bpm: 190.0,
            selected_downbeat_seconds: Some(0.125),
            selected_phase_anchor_seconds: Some(0.125),
            ..Default::default()
        };
        let guided = classify_timing_reachability_with_tempo_guided(
            190.0,
            None,
            Some(141.509_45),
            SourceTimingProbeReadinessStatus::Weak,
            true,
            "manual_confirm_only",
            Some(evidence),
        );
        assert_eq!(
            guided.confirmation_route,
            "source_derived_downbeat_at_external_tempo"
        );
        assert!(guided.product_projection_allowed);

        let mut product = primary_timing(TimingHypothesisKind::TempoGuided, 190.0);
        product.hypotheses[0].bar_grid = vec![riotbox_core::source_graph::BarSpan {
            bar_index: 1,
            start_seconds: 0.125,
            end_seconds: 1.388,
            downbeat_confidence: 0.9,
            phrase_index: None,
        }];
        let mut report =
            W30ReachabilityPreflightReport::from_timing(Path::new("guided.wav"), guided);
        report.record_product_timing(&product);

        assert_eq!(
            report.timing.product_graph_matches_confirmation_route,
            Some(true)
        );
        assert!(report.timing.product_projection_allowed);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn source_phase_rejection_preserves_existing_mismatch_failure() {
        let rejected = classify_timing_reachability_with_tempo_guided(
            190.0,
            None,
            Some(141.509_45),
            SourceTimingProbeReadinessStatus::Weak,
            true,
            "manual_confirm_only",
            Some(TempoGuidedTimingEvidence {
                decision: TempoGuidedTimingDecision::AmbiguousPhase,
                requested_bpm: 190.0,
                ..Default::default()
            }),
        );

        assert_eq!(rejected.confirmation_route, "explicit_bpm_mismatch");
        assert!(!rejected.product_projection_allowed);
        assert_eq!(
            rejected
                .tempo_guided_evidence
                .as_ref()
                .map(|evidence| evidence.decision),
            Some(TempoGuidedTimingDecision::AmbiguousPhase)
        );
    }

    #[test]
    fn h14_timing_match_still_rejects_non_applicable_hard_recipe() {
        let timing = timing_report(Some(139.751_56), 140.0);
        assert!(timing.product_projection_allowed);
        let mut report = W30ReachabilityPreflightReport::from_timing(Path::new("h14.wav"), timing);
        let state = W30ResampleTapState {
            variation: W30ResampleTapVariation::HardDamage,
            hard_intent: Some(W30HardIntent::Impact),
            hard_intent_outcome: W30HardIntentOutcome::RealizedImpact,
            hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
            tempo_bpm: 140.0,
            ..Default::default()
        };

        report.record_projection(&state);

        assert!(!report.candidate_wav_generation_eligible_after_preflight);
        assert_eq!(
            report.blockers,
            ["exact_hit_shaper_calibration_not_applicable"]
        );
    }

    #[test]
    fn exact_product_hit_shaper_state_allows_candidate_generation() {
        let timing = timing_report(Some(130.4), 130.0);
        let mut report =
            W30ReachabilityPreflightReport::from_timing(Path::new("development-loop.wav"), timing);
        report.timing.product_graph_matches_confirmation_route = Some(true);

        report.record_projection(&applicable_hit_shaper_state());

        assert!(report.candidate_wav_generation_eligible_after_preflight);
        assert!(report.blockers.is_empty());
        let projection = report.projection.expect("projection");
        assert!(projection.candidate_requirement_satisfied);
        assert_eq!(projection.requested_intent, "impact");
        assert_eq!(projection.intent_outcome, "realized_impact");
        assert_eq!(projection.low_impact_recipe, "source_impact_shaper_v4");
        assert_eq!(projection.low_impact_role, "transient_impact");
        assert_eq!(projection.low_impact_decision, "source_hit_selected");
        assert_eq!(projection.low_impact_candidate_count, 5);
        assert_eq!(projection.low_impact_selected_slot, 3);
        assert_eq!(projection.low_impact_selected_onset_cursor, 1_234);
        assert_eq!(projection.low_impact_attack_window_proxy_frames, 48);
        assert_eq!(projection.low_impact_body_window_proxy_frames, 96);
        assert_eq!(projection.low_band_attack_share, 0.24);
        assert_eq!(projection.low_band_attack_share_min, 0.18);
        assert_eq!(projection.low_band_attack_over_body, 1.65);
        assert_eq!(projection.low_band_attack_over_body_min, 1.40);
        assert_eq!(projection.low_band_attack_over_source, 0.42);
        assert_eq!(projection.low_band_attack_over_source_min, 0.30);
    }

    #[test]
    fn exact_hit_shaper_without_performer_impact_intent_cannot_generate_candidate() {
        let timing = timing_report(Some(130.4), 130.0);
        let mut report =
            W30ReachabilityPreflightReport::from_timing(Path::new("legacy-loop.wav"), timing);
        report.timing.product_graph_matches_confirmation_route = Some(true);
        let mut state = applicable_hit_shaper_state();
        state.hard_intent = Some(W30HardIntent::LegacyAuto);
        state.hard_intent_outcome = W30HardIntentOutcome::LegacyAuto;

        report.record_projection(&state);

        assert!(!report.candidate_wav_generation_eligible_after_preflight);
        assert_eq!(report.blockers, ["performer_impact_intent_not_committed"]);
        let projection = report.projection.expect("projection");
        assert_eq!(projection.requested_intent, "legacy_auto");
        assert_eq!(projection.intent_outcome, "legacy_auto");
        assert!(!projection.candidate_requirement_satisfied);
    }

    #[test]
    fn recipe_without_matching_typed_role_and_decision_is_not_applicable() {
        let mut state = applicable_hit_shaper_state();
        state.hard_low_impact.role = W30ResampleLowImpactRole::Unassigned;
        state.hard_low_impact.decision = W30ResampleLowImpactDecision::NotEvaluated;

        assert!(!state.exact_hit_shaper_calibration_applicable());
    }

    #[test]
    fn evaluated_fail_closed_rejection_reports_prior_applicability() {
        let timing = timing_report(Some(130.4), 130.0);
        let mut report =
            W30ReachabilityPreflightReport::from_timing(Path::new("rejected-loop.wav"), timing);
        report.timing.product_graph_matches_confirmation_route = Some(true);
        let mut state = applicable_hit_shaper_state();
        state.hard_policy = W30ResampleTapHardPolicy::Unavailable;
        state.hard_intent_outcome = W30HardIntentOutcome::SourceMismatch;
        state.hard_low_impact.recipe = W30ResampleLowImpactRecipe::SourceAlignedImpactV5;
        state.hard_low_impact.decision = W30ResampleLowImpactDecision::ExactCallbackRejected;
        state.hard_calibration.exact_callback_evaluated = true;
        state.hard_calibration.exact_callback_calibrated = false;

        report.record_projection(&state);

        assert!(!report.candidate_wav_generation_eligible_after_preflight);
        assert_eq!(report.blockers, ["exact_hit_shaper_calibration_rejected"]);
        let projection = report.projection.expect("projection");
        assert!(projection.exact_callback_calibration_applicable);
        assert!(projection.exact_callback_evaluated);
        assert!(!projection.exact_callback_calibrated);
        assert!(!projection.candidate_requirement_satisfied);
    }
}
