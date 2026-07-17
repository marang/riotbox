use crate::{
    ids::SourceId,
    session::{Mc202SourcePhraseCandidateFamilyState, SessionFile},
    source_graph::{
        EnergyClass, QualityClass, SourceGraph, section_for_projected_scene,
        section_for_transport_bar,
    },
    transport::{
        DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState, TransportGridPosition,
    },
    view::jam::source_timing_confirmation_matches_graph,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LivePerformanceMc202Intent {
    BassPressure,
    Punctuate,
    Instigate,
    StayOut,
}

impl LivePerformanceMc202Intent {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BassPressure => "bass_pressure",
            Self::Punctuate => "punctuate",
            Self::Instigate => "instigate",
            Self::StayOut => "stay_out",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LivePerformanceLead {
    W30Hook,
    Tr909Pressure,
}

impl LivePerformanceLead {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::W30Hook => "w30_hook",
            Self::Tr909Pressure => "tr909_pressure",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LivePerformanceBassOwner {
    Mc202,
    Unassigned,
}

impl LivePerformanceBassOwner {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mc202 => "mc202",
            Self::Unassigned => "unassigned",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LivePerformancePolicy {
    pub source_id: SourceId,
    /// Confirmed source-bar downbeat expressed in the zero-based Session transport cursor.
    ///
    /// Runtime renderers subtract this phase anchor before evaluating bar-local vocabulary.
    /// `None` preserves legacy zero-phase behavior when the confirmed graph cannot resolve an
    /// evidenced bar anchor.
    pub source_bar_grid_anchor_beat_cursor: Option<u64>,
    pub lead: LivePerformanceLead,
    pub bass_owner: LivePerformanceBassOwner,
    pub mc202_intent: LivePerformanceMc202Intent,
    pub w30_music_level: f32,
    /// Trusted current-section transient evidence used to derive TR-909 pressure, when present.
    pub source_transient_backbeat_evidence: Option<f32>,
    pub tr909_drum_level: f32,
    pub tr909_slam_floor: f32,
    pub mc202_music_level: f32,
    pub mc202_touch_floor: f32,
}

#[must_use]
pub fn derive_live_performance_policy(
    session: &SessionFile,
    graph: &SourceGraph,
) -> Option<LivePerformancePolicy> {
    let confirmed = source_timing_confirmation_matches_graph(graph, session);
    let has_break_window = graph.analysis_summary.break_rebuild_potential == QualityClass::High
        && (graph.loop_candidate_count() > 0 || graph.analysis_summary.loop_candidate_count > 0)
        && graph.sections.iter().any(|section| {
            matches!(
                section.energy_class,
                EnergyClass::Medium | EnergyClass::High | EnergyClass::Peak
            )
        });
    if !confirmed || !has_break_window {
        return None;
    }

    let source_plan = session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .filter(|plan| plan.source_id == graph.source.source_id)?;
    let section_ownership_is_current = source_plan.source_section_id.as_ref().is_none_or(|owner| {
        current_source_section(session, graph).is_some_and(|section| section.section_id == *owner)
    });
    let mc202_intent = if !source_plan.is_source_derived() || !section_ownership_is_current {
        LivePerformanceMc202Intent::StayOut
    } else {
        match source_plan.candidate_family {
            Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove) => {
                LivePerformanceMc202Intent::BassPressure
            }
            Some(
                Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
                | Mc202SourcePhraseCandidateFamilyState::CallBackStab
                | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer,
            ) => LivePerformanceMc202Intent::Punctuate,
            Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator) => {
                LivePerformanceMc202Intent::Instigate
            }
            Some(
                Mc202SourcePhraseCandidateFamilyState::StayOut
                | Mc202SourcePhraseCandidateFamilyState::FallbackControl,
            ) => LivePerformanceMc202Intent::StayOut,
            None => LivePerformanceMc202Intent::StayOut,
        }
    };
    let expression = section_ownership_is_current
        .then_some(source_plan.source_expression.as_ref())
        .flatten();
    let bass_pressure = expression.map_or(0.0, |value| value.bass_pressure.clamp(0.0, 1.0));
    let source_transient_backbeat_evidence =
        expression.map(|value| value.transient_backbeat.clamp(0.0, 1.0));
    let transient_backbeat = source_transient_backbeat_evidence.unwrap_or(0.55);
    let lead = if transient_backbeat >= 0.72 {
        LivePerformanceLead::Tr909Pressure
    } else {
        LivePerformanceLead::W30Hook
    };
    let mc202_music_level = match mc202_intent {
        LivePerformanceMc202Intent::BassPressure => 0.76 + bass_pressure * 0.16,
        LivePerformanceMc202Intent::Punctuate => 0.62,
        LivePerformanceMc202Intent::Instigate => 0.70,
        LivePerformanceMc202Intent::StayOut => 0.0,
    };
    let mc202_touch_floor = match mc202_intent {
        LivePerformanceMc202Intent::BassPressure => 0.82,
        LivePerformanceMc202Intent::Punctuate => 0.72,
        LivePerformanceMc202Intent::Instigate => 0.82,
        LivePerformanceMc202Intent::StayOut => 0.0,
    };
    let bass_owner = if mc202_intent == LivePerformanceMc202Intent::BassPressure {
        LivePerformanceBassOwner::Mc202
    } else {
        LivePerformanceBassOwner::Unassigned
    };

    Some(LivePerformancePolicy {
        source_id: graph.source.source_id.clone(),
        source_bar_grid_anchor_beat_cursor: graph
            .timing
            .primary_hypothesis()
            .and_then(|hypothesis| hypothesis.transport_bar_grid_anchor())
            .map(|anchor| anchor.beat_cursor),
        lead,
        bass_owner,
        mc202_intent,
        w30_music_level: if mc202_intent == LivePerformanceMc202Intent::BassPressure {
            0.60
        } else {
            0.64
        },
        source_transient_backbeat_evidence,
        tr909_drum_level: 0.68 + transient_backbeat * 0.16,
        tr909_slam_floor: 0.54 + transient_backbeat * 0.16,
        mc202_music_level,
        mc202_touch_floor,
    })
}

fn current_source_section<'a>(
    session: &SessionFile,
    graph: &'a SourceGraph,
) -> Option<&'a crate::source_graph::Section> {
    let scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref());
    if let Some(section) = scene.and_then(|scene| section_for_projected_scene(graph, scene)) {
        return Some(section);
    }

    let beats_per_bar = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| u64::from(hypothesis.meter.beats_per_bar))
        .or_else(|| {
            graph
                .timing
                .meter_hint
                .as_ref()
                .map(|meter| u64::from(meter.beats_per_bar))
        })
        .filter(|beats| *beats > 0)
        .unwrap_or(DEFAULT_BEATS_PER_BAR);
    let grid = TransportGridPosition::from_zero_based_position_beats(
        session.runtime_state.transport.position_beats,
        beats_per_bar,
        DEFAULT_BARS_PER_PHRASE,
    );
    section_for_transport_bar(
        graph,
        &TransportClockState {
            is_playing: session.runtime_state.transport.is_playing,
            position_beats: session.runtime_state.transport.position_beats,
            beat_index: grid.beat_cursor,
            bar_index: grid.bar_index,
            phrase_index: grid.phrase_index,
            current_scene: session.runtime_state.transport.current_scene.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ids::{ActionId, SceneId, SectionId, SourceId},
        session::{
            Mc202RoleState, Mc202SourcePhraseExpressionState, Mc202SourcePhraseNoteBudgetState,
            Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState,
            SourceTimingGridConfirmationState,
        },
        source_graph::{
            DecodeProfile, GraphProvenance, Section, SectionLabelHint, SourceDescriptor,
        },
    };

    #[test]
    fn dense_break_policy_stays_unavailable_without_confirmed_timing() {
        let (session, graph) = dense_break_context();

        assert!(derive_live_performance_policy(&session, &graph).is_none());
    }

    #[test]
    fn dense_break_policy_rejects_stale_confirmation_for_same_source() {
        let (mut session, mut graph) = dense_break_context();
        graph.timing.primary_hypothesis_id = Some("grid-current".into());
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-stale".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });

        assert!(derive_live_performance_policy(&session, &graph).is_none());
    }

    #[test]
    fn dense_break_policy_stays_unavailable_without_committed_source_plan() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });

        assert!(derive_live_performance_policy(&session, &graph).is_none());
    }

    #[test]
    fn source_derived_pressure_plan_opens_physical_all_lane_levels() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        session.runtime_state.lane_state.mc202.source_phrase_plan =
            Some(pressure_source_plan(graph.source.source_id.clone()));

        let policy = derive_live_performance_policy(&session, &graph).expect("dense policy");

        assert_eq!(
            policy.mc202_intent,
            LivePerformanceMc202Intent::BassPressure
        );
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Mc202);
        assert_eq!(policy.source_transient_backbeat_evidence, Some(0.78));
        assert!(policy.tr909_drum_level >= 0.75);
        assert!(policy.tr909_slam_floor >= 0.60);
        assert!(policy.mc202_music_level >= 0.80);
        assert!(policy.mc202_touch_floor >= 0.82);
        assert_eq!(policy.w30_music_level, 0.60);
    }

    #[test]
    fn typed_plan_from_previous_scene_cannot_claim_current_bass_ownership_or_floors() {
        let (mut session, mut graph) = dense_break_context();
        graph.sections.push(Section {
            section_id: SectionId::from("break-2"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 8.0,
            end_seconds: 16.0,
            bar_start: 5,
            bar_end: 8,
            energy_class: EnergyClass::Medium,
            confidence: 0.9,
            tags: vec!["break".into()],
        });
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.source_section_id = Some(SectionId::from("drop-1"));
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let policy = derive_live_performance_policy(&session, &graph)
            .expect("section-mismatched dense policy stays explicit");

        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::StayOut);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(policy.mc202_music_level, 0.0);
        assert_eq!(policy.mc202_touch_floor, 0.0);
        assert!((policy.tr909_drum_level - 0.768).abs() < 1.0e-6);
        assert!((policy.tr909_slam_floor - 0.628).abs() < 1.0e-6);
        assert_eq!(policy.w30_music_level, 0.64);
    }

    #[test]
    fn committed_fallback_plan_stays_out_instead_of_reviving_requested_role() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FallbackControl);
        plan.fallback_reason = Some("source_evidence_untrusted".into());
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let policy = derive_live_performance_policy(&session, &graph).expect("stay-out policy");

        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::StayOut);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(policy.mc202_music_level, 0.0);
        assert_eq!(policy.mc202_touch_floor, 0.0);
    }

    #[test]
    fn legacy_plan_without_candidate_family_cannot_claim_bass_from_requested_role() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = None;
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let policy = derive_live_performance_policy(&session, &graph).expect("legacy policy");

        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::StayOut);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
    }

    #[test]
    fn callback_candidate_resolves_to_answer_even_when_pressure_was_requested() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::CallBackStab);
        plan.source_expression
            .as_mut()
            .expect("source expression")
            .bass_pressure = 0.86;
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let policy = derive_live_performance_policy(&session, &graph).expect("dense policy");

        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::Punctuate);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(policy.mc202_music_level, 0.62);
        assert_eq!(policy.mc202_touch_floor, 0.72);
        assert_eq!(policy.w30_music_level, 0.64);
    }

    #[test]
    fn live_policy_rederives_identically_from_persisted_product_truth() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::CallBackStab);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);
        let committed_policy =
            derive_live_performance_policy(&session, &graph).expect("committed dense policy");

        let restored_session: SessionFile = serde_json::from_str(
            &serde_json::to_string(&session).expect("serialize live-policy session truth"),
        )
        .expect("restore live-policy session truth");
        let restored_graph: SourceGraph = serde_json::from_str(
            &serde_json::to_string(&graph).expect("serialize live-policy source graph"),
        )
        .expect("restore live-policy source graph");
        let restored_policy = derive_live_performance_policy(&restored_session, &restored_graph)
            .expect("restored dense policy");

        assert_eq!(restored_policy, committed_policy);
        assert_eq!(
            restored_policy.bass_owner,
            LivePerformanceBassOwner::Unassigned
        );
        assert_eq!(
            restored_policy.mc202_intent,
            LivePerformanceMc202Intent::Punctuate
        );
    }

    fn dense_break_context() -> (SessionFile, SourceGraph) {
        let source_id = SourceId::from("src-dense");
        let source = SourceDescriptor {
            source_id,
            path: "dense.wav".into(),
            content_hash: "dense-hash".into(),
            duration_seconds: 8.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        };
        let provenance = GraphProvenance {
            sidecar_version: "test".into(),
            provider_set: vec!["test".into()],
            generated_at: "2026-07-12".into(),
            source_hash: "dense-hash".into(),
            analysis_seed: 1,
            run_notes: None,
        };
        let mut graph = SourceGraph::new(source, provenance);
        graph.timing.primary_hypothesis_id = Some("grid-1".into());
        graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        graph.analysis_summary.loop_candidate_count = 1;
        graph.sections.push(Section {
            section_id: SectionId::from("drop-1"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 8.0,
            bar_start: 1,
            bar_end: 4,
            energy_class: EnergyClass::Peak,
            confidence: 0.9,
            tags: vec!["dense_break".into()],
        });
        (SessionFile::new("session", "test", "2026-07-12"), graph)
    }

    fn pressure_source_plan(source_id: SourceId) -> Mc202SourcePhrasePlanState {
        Mc202SourcePhrasePlanState {
            source_id,
            source_section_id: None,
            phrase_slot: Mc202SourcePhraseSlotState {
                phrase_index: 0,
                start_bar: 1,
                end_bar: 4,
            },
            source_expression: Some(Mc202SourcePhraseExpressionState {
                low_pressure_contour: 0.82,
                bass_pressure: 0.86,
                transient_backbeat: 0.78,
                offbeat_answer_space: 0.42,
                phrase_density: 0.58,
                hook_restraint: 0.35,
                stab_bite: 0.38,
                stay_out_pressure: 0.18,
                confidence: 0.88,
                provenance_refs: vec!["source-map".into()],
            }),
            role: Mc202RoleState::Pressure,
            rhythm_cells: [
                Some(-12),
                None,
                None,
                None,
                Some(-7),
                None,
                None,
                None,
                Some(-10),
                None,
                None,
                None,
                Some(-5),
                None,
                None,
                None,
            ],
            note_budget: Mc202SourcePhraseNoteBudgetState::Balanced,
            touch: 0.84,
            confidence: 0.88,
            candidate_family: Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove),
            candidate_count: 3,
            rejected_candidate_count: 2,
            candidate_provenance_refs: vec!["source-map".into()],
            candidate_scorecards: Vec::new(),
            phrase_memory_distance: 0.6,
            fallback_reason: None,
        }
    }
}
