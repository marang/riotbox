use crate::{
    ids::SourceId,
    session::{Mc202SourcePhraseCandidateFamilyState, SessionFile},
    source_graph::{
        EnergyClass, PhraseAudioFeatures, QualityClass, SourceGraph, section_for_projected_scene,
        section_for_transport_bar,
    },
    tr909_policy::{Tr909PatternAdoptionPolicy, Tr909PhraseVariationPolicy},
    transport::{
        DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState, TransportGridPosition,
    },
    view::jam::source_timing_confirmation_matches_graph,
};

/// Minimum normalized contrast required before phrase-audio evidence may move a source out of
/// the proven dense-break default.
///
/// This is a classification margin, not a loudness or QA threshold. The controlled source
/// matrix constrains it: the accepted Beat03 dense break stays inside the neutral band, while
/// the trusted RushArp tonal source and BeatC sparse drum source clear opposite sides.
pub const LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN: f32 = 0.10;
const DENSE_TR909_LEAD_TRANSIENT_THRESHOLD: f32 = 0.72;
const DEFAULT_TRANSIENT_BACKBEAT: f32 = 0.55;
const TONAL_W30_MUSIC_LEVEL: f32 = 0.70;
const TONAL_TR909_DRUM_BASE: f32 = 0.50;
const TONAL_TR909_DRUM_TRANSIENT_SCALE: f32 = 0.12;
const TONAL_TR909_SLAM_BASE: f32 = 0.28;
const TONAL_TR909_SLAM_TRANSIENT_SCALE: f32 = 0.10;
const SPARSE_W30_MUSIC_LEVEL: f32 = 0.38;
const SPARSE_TR909_DRUM_BASE: f32 = 0.84;
const SPARSE_TR909_DRUM_TRANSIENT_SCALE: f32 = 0.15;
const SPARSE_TR909_SLAM_BASE: f32 = 0.65;
const SPARSE_TR909_SLAM_TRANSIENT_SCALE: f32 = 0.15;
const SPARSE_MC202_PUNCTUATE_MUSIC_LEVEL: f32 = 0.46;
const SPARSE_MC202_PUNCTUATE_TOUCH_FLOOR: f32 = 0.68;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LivePerformanceCharacter {
    DenseBreak,
    TonalHook,
    SparsePressure,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LivePerformanceDestructiveIntent {
    PitchDrag,
    TransientBite,
}

impl LivePerformanceDestructiveIntent {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PitchDrag => "pitch_drag",
            Self::TransientBite => "transient_bite",
        }
    }
}

impl LivePerformanceCharacter {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::DenseBreak => "dense_break",
            Self::TonalHook => "tonal_hook",
            Self::SparsePressure => "sparse_pressure",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LivePerformanceCharacterEvidence {
    pub phrase_index: u32,
    pub spectral_brightness: f32,
    pub low_mid_ratio: f32,
    pub offbeat_onset_density: f32,
    pub hook_restraint_hint: f32,
    pub confidence: f32,
}

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
    pub character: LivePerformanceCharacter,
    pub character_evidence: Option<LivePerformanceCharacterEvidence>,
    pub destructive_intent: LivePerformanceDestructiveIntent,
    pub lead: LivePerformanceLead,
    pub bass_owner: LivePerformanceBassOwner,
    pub mc202_intent: LivePerformanceMc202Intent,
    pub w30_music_level: f32,
    /// Trusted current-section transient evidence used to derive TR-909 pressure, when present.
    pub source_transient_backbeat_evidence: Option<f32>,
    pub tr909_drum_level: f32,
    pub tr909_slam_floor: f32,
    /// Source-character defaults for the held live state. Explicit fills and scene movement keep
    /// their committed vocabulary and take precedence in the app projection.
    pub tr909_pattern_adoption: Option<Tr909PatternAdoptionPolicy>,
    pub tr909_phrase_variation: Option<Tr909PhraseVariationPolicy>,
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
    let source_mc202_intent = if !source_plan.is_source_derived() || !section_ownership_is_current {
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
    let character_evidence = section_ownership_is_current
        .then(|| character_evidence(graph, source_plan))
        .flatten();
    let character = character_evidence
        .as_ref()
        .map_or(LivePerformanceCharacter::DenseBreak, classify_character);
    // Preserve explicit bass ownership and source-selected answer/stay-out families. Character
    // policy only restrains the generic fill-pickup result that otherwise collapsed all three
    // trusted source families into the same live behavior.
    let mc202_intent = match (character, source_mc202_intent) {
        (LivePerformanceCharacter::TonalHook, LivePerformanceMc202Intent::Instigate) => {
            LivePerformanceMc202Intent::StayOut
        }
        (LivePerformanceCharacter::SparsePressure, LivePerformanceMc202Intent::Instigate) => {
            LivePerformanceMc202Intent::Punctuate
        }
        (_, source_intent) => source_intent,
    };
    let bass_pressure = expression.map_or(0.0, |value| value.bass_pressure.clamp(0.0, 1.0));
    let source_transient_backbeat_evidence =
        expression.map(|value| value.transient_backbeat.clamp(0.0, 1.0));
    let transient_backbeat =
        source_transient_backbeat_evidence.unwrap_or(DEFAULT_TRANSIENT_BACKBEAT);
    let lead = match character {
        LivePerformanceCharacter::TonalHook => LivePerformanceLead::W30Hook,
        LivePerformanceCharacter::SparsePressure => LivePerformanceLead::Tr909Pressure,
        LivePerformanceCharacter::DenseBreak => {
            if transient_backbeat >= DENSE_TR909_LEAD_TRANSIENT_THRESHOLD {
                LivePerformanceLead::Tr909Pressure
            } else {
                LivePerformanceLead::W30Hook
            }
        }
    };
    let destructive_intent = match character {
        LivePerformanceCharacter::SparsePressure => LivePerformanceDestructiveIntent::TransientBite,
        LivePerformanceCharacter::DenseBreak | LivePerformanceCharacter::TonalHook => {
            LivePerformanceDestructiveIntent::PitchDrag
        }
    };
    let mc202_music_level = match mc202_intent {
        LivePerformanceMc202Intent::BassPressure => 0.76 + bass_pressure * 0.16,
        LivePerformanceMc202Intent::Punctuate
            if character == LivePerformanceCharacter::SparsePressure =>
        {
            SPARSE_MC202_PUNCTUATE_MUSIC_LEVEL
        }
        LivePerformanceMc202Intent::Punctuate => 0.62,
        // A fill-pickup is an accent, not a bass owner. Its higher touch floor supplies the
        // attack; a lower bus allocation leaves exact-mix headroom for the source-backed W-30
        // hook and physical TR-909 downbeat.
        LivePerformanceMc202Intent::Instigate => 0.50,
        LivePerformanceMc202Intent::StayOut => 0.0,
    };
    let mc202_touch_floor = match mc202_intent {
        LivePerformanceMc202Intent::BassPressure => 0.82,
        LivePerformanceMc202Intent::Punctuate
            if character == LivePerformanceCharacter::SparsePressure =>
        {
            SPARSE_MC202_PUNCTUATE_TOUCH_FLOOR
        }
        LivePerformanceMc202Intent::Punctuate => 0.72,
        LivePerformanceMc202Intent::Instigate => 0.82,
        LivePerformanceMc202Intent::StayOut => 0.0,
    };
    let bass_owner = if mc202_intent == LivePerformanceMc202Intent::BassPressure {
        LivePerformanceBassOwner::Mc202
    } else {
        LivePerformanceBassOwner::Unassigned
    };

    let nominal_w30_music_level: f32 = match (character, mc202_intent) {
        (_, LivePerformanceMc202Intent::BassPressure) => 0.60,
        (LivePerformanceCharacter::TonalHook, _) => TONAL_W30_MUSIC_LEVEL,
        (LivePerformanceCharacter::SparsePressure, _) => SPARSE_W30_MUSIC_LEVEL,
        (LivePerformanceCharacter::DenseBreak, _) => 0.64,
    };
    let (tr909_drum_level, tr909_slam_floor, tr909_pattern_adoption, tr909_phrase_variation) =
        match character {
            LivePerformanceCharacter::DenseBreak => (
                0.68 + transient_backbeat * 0.16,
                0.54 + transient_backbeat * 0.16,
                None,
                None,
            ),
            // Let the captured tonal phrase lead. A steady anchor remains audible, while the
            // reduced drum/slam allocation and MC-202 stay-out prevent pitch collision.
            LivePerformanceCharacter::TonalHook => (
                TONAL_TR909_DRUM_BASE + transient_backbeat * TONAL_TR909_DRUM_TRANSIENT_SCALE,
                TONAL_TR909_SLAM_BASE + transient_backbeat * TONAL_TR909_SLAM_TRANSIENT_SCALE,
                Some(Tr909PatternAdoptionPolicy::SupportPulse),
                Some(Tr909PhraseVariationPolicy::PhraseAnchor),
            ),
            // Sparse drum material receives a harder mainline anchor, but PhraseAnchor avoids
            // turning every held bar into a fill. The MC-202 becomes a quiet punctuation layer,
            // never an undeclared bass owner.
            LivePerformanceCharacter::SparsePressure => (
                SPARSE_TR909_DRUM_BASE + transient_backbeat * SPARSE_TR909_DRUM_TRANSIENT_SCALE,
                SPARSE_TR909_SLAM_BASE + transient_backbeat * SPARSE_TR909_SLAM_TRANSIENT_SCALE,
                Some(Tr909PatternAdoptionPolicy::MainlineDrive),
                Some(Tr909PhraseVariationPolicy::PhraseAnchor),
            ),
        };

    Some(LivePerformancePolicy {
        source_id: graph.source.source_id.clone(),
        source_bar_grid_anchor_beat_cursor: graph
            .timing
            .primary_hypothesis()
            .and_then(|hypothesis| hypothesis.transport_bar_grid_anchor())
            .map(|anchor| anchor.beat_cursor),
        character,
        character_evidence,
        destructive_intent,
        lead,
        bass_owner,
        mc202_intent,
        // The policy owns the musical balance, while the committed Session mixer remains an
        // explicit headroom ceiling. Ignoring that ceiling made a named preset unable to protect
        // the exact live mixer from source-dependent hot peaks.
        w30_music_level: session.runtime_state.style.active_preset.map_or(
            nominal_w30_music_level,
            |_| {
                nominal_w30_music_level.min(
                    session
                        .runtime_state
                        .mixer_state
                        .music_level
                        .clamp(0.0, 1.0),
                )
            },
        ),
        source_transient_backbeat_evidence,
        tr909_drum_level,
        tr909_slam_floor,
        tr909_pattern_adoption,
        tr909_phrase_variation,
        mc202_music_level,
        mc202_touch_floor,
    })
}

fn character_evidence(
    graph: &SourceGraph,
    source_plan: &crate::session::Mc202SourcePhrasePlanState,
) -> Option<LivePerformanceCharacterEvidence> {
    let slot = &source_plan.phrase_slot;
    let measured = || {
        graph
            .phrase_audio_features
            .iter()
            .filter(|feature| feature.has_measured_evidence())
    };
    measured()
        .filter(|feature| feature.phrase_index == slot.phrase_index)
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        .or_else(|| {
            measured()
                .filter(|feature| {
                    feature.start_bar <= slot.end_bar && feature.end_bar >= slot.start_bar
                })
                .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        })
        .map(character_evidence_from_features)
}

fn character_evidence_from_features(
    features: &PhraseAudioFeatures,
) -> LivePerformanceCharacterEvidence {
    LivePerformanceCharacterEvidence {
        phrase_index: features.phrase_index,
        spectral_brightness: features.spectral_brightness.clamp(0.0, 1.0),
        low_mid_ratio: features.low_mid_ratio.clamp(0.0, 1.0),
        offbeat_onset_density: features.offbeat_onset_density.clamp(0.0, 1.0),
        hook_restraint_hint: features.hook_restraint_hint.clamp(0.0, 1.0),
        confidence: features.confidence.clamp(0.0, 1.0),
    }
}

fn classify_character(evidence: &LivePerformanceCharacterEvidence) -> LivePerformanceCharacter {
    let tonal_spectral_contrast = evidence.spectral_brightness - evidence.low_mid_ratio;
    let tonal_hook_contrast = evidence.hook_restraint_hint - evidence.low_mid_ratio;
    if tonal_spectral_contrast >= LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN
        && tonal_hook_contrast >= LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN
    {
        return LivePerformanceCharacter::TonalHook;
    }

    let sparse_body_contrast = evidence.low_mid_ratio - evidence.spectral_brightness;
    let sparse_space_contrast = evidence.hook_restraint_hint - evidence.offbeat_onset_density;
    if sparse_body_contrast >= LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN
        && sparse_space_contrast >= LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN
    {
        return LivePerformanceCharacter::SparsePressure;
    }

    LivePerformanceCharacter::DenseBreak
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
        style::PerformancePresetId,
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
    fn active_preset_mixer_caps_the_policy_w30_level_for_live_headroom() {
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
        PerformancePresetId::FeralBreakAlphaV2.apply_to_session(&mut session);

        let policy = derive_live_performance_policy(&session, &graph).expect("dense policy");

        assert_eq!(
            policy.w30_music_level,
            PerformancePresetId::FeralBreakAlphaV2
                .definition()
                .mixer_state
                .music_level
        );
        assert!(policy.w30_music_level < 0.64);
    }

    #[test]
    fn fill_pickup_instigator_keeps_headroom_and_cannot_claim_bass() {
        let (mut session, graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let policy = derive_live_performance_policy(&session, &graph).expect("dense policy");

        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::Instigate);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(policy.mc202_music_level, 0.50);
        assert_eq!(policy.mc202_touch_floor, 0.82);
    }

    #[test]
    fn measured_tonal_character_promotes_w30_and_restrains_generic_instigator() {
        let (mut session, mut graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);
        graph
            .phrase_audio_features
            .push(phrase_features(0.34, 0.88, 0.43, 0.61));

        let policy = derive_live_performance_policy(&session, &graph).expect("tonal policy");

        assert_eq!(policy.character, LivePerformanceCharacter::TonalHook);
        assert_eq!(
            policy.destructive_intent,
            LivePerformanceDestructiveIntent::PitchDrag
        );
        assert_eq!(policy.lead, LivePerformanceLead::W30Hook);
        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::StayOut);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(
            policy.tr909_pattern_adoption,
            Some(Tr909PatternAdoptionPolicy::SupportPulse)
        );
        assert_eq!(
            policy.tr909_phrase_variation,
            Some(Tr909PhraseVariationPolicy::PhraseAnchor)
        );
        assert!(policy.tr909_drum_level < 0.70);
        assert_eq!(policy.mc202_music_level, 0.0);
    }

    #[test]
    fn measured_sparse_character_anchors_drums_without_claiming_bass_or_dense_fill() {
        let (mut session, mut graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);
        graph
            .phrase_audio_features
            .push(phrase_features(0.70, 0.39, 0.20, 0.38));

        let policy = derive_live_performance_policy(&session, &graph).expect("sparse policy");

        assert_eq!(policy.character, LivePerformanceCharacter::SparsePressure);
        assert_eq!(
            policy.destructive_intent,
            LivePerformanceDestructiveIntent::TransientBite
        );
        assert_eq!(policy.lead, LivePerformanceLead::Tr909Pressure);
        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::Punctuate);
        assert_eq!(policy.bass_owner, LivePerformanceBassOwner::Unassigned);
        assert_eq!(
            policy.tr909_pattern_adoption,
            Some(Tr909PatternAdoptionPolicy::MainlineDrive)
        );
        assert_eq!(
            policy.tr909_phrase_variation,
            Some(Tr909PhraseVariationPolicy::PhraseAnchor)
        );
        assert!(policy.tr909_drum_level > 0.80);
        assert_eq!(policy.mc202_music_level, 0.46);
    }

    #[test]
    fn neutral_measured_character_preserves_human_passed_dense_policy() {
        let (mut session, mut graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);
        graph
            .phrase_audio_features
            .push(phrase_features(0.58, 0.58, 0.13, 0.47));

        let policy = derive_live_performance_policy(&session, &graph).expect("dense policy");

        assert_eq!(policy.character, LivePerformanceCharacter::DenseBreak);
        assert_eq!(
            policy.destructive_intent,
            LivePerformanceDestructiveIntent::PitchDrag
        );
        assert_eq!(policy.lead, LivePerformanceLead::Tr909Pressure);
        assert_eq!(policy.mc202_intent, LivePerformanceMc202Intent::Instigate);
        assert_eq!(policy.tr909_pattern_adoption, None);
        assert_eq!(policy.tr909_phrase_variation, None);
        assert!((policy.tr909_drum_level - 0.8048).abs() < 1.0e-6);
        assert!((policy.tr909_slam_floor - 0.6648).abs() < 1.0e-6);
    }

    #[test]
    fn exact_phrase_character_evidence_beats_higher_confidence_overlap() {
        let (mut session, mut graph) = dense_break_context();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: graph.source.source_id.clone(),
                hypothesis_id: Some("grid-1".into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 100,
            });
        let mut plan = pressure_source_plan(graph.source.source_id.clone());
        plan.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator);
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);

        let mut exact_tonal = phrase_features(0.34, 0.88, 0.43, 0.61);
        exact_tonal.phrase_index = 0;
        exact_tonal.confidence = 0.40;
        let overlapping_sparse = phrase_features(0.70, 0.39, 0.20, 0.38);
        graph.phrase_audio_features = vec![overlapping_sparse, exact_tonal];

        let policy = derive_live_performance_policy(&session, &graph).expect("tonal policy");

        assert_eq!(policy.character, LivePerformanceCharacter::TonalHook);
        assert_eq!(
            policy
                .character_evidence
                .map(|evidence| evidence.phrase_index),
            Some(0)
        );
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

    fn phrase_features(
        low_mid_ratio: f32,
        spectral_brightness: f32,
        offbeat_onset_density: f32,
        hook_restraint_hint: f32,
    ) -> PhraseAudioFeatures {
        PhraseAudioFeatures {
            phrase_index: 1,
            start_seconds: 0.0,
            end_seconds: 8.0,
            start_bar: 1,
            end_bar: 4,
            low_band_rms: 0.1,
            low_mid_ratio,
            low_band_movement: 0.1,
            transient_density: 1.0,
            offbeat_onset_density,
            spectral_roughness: 0.05,
            spectral_brightness,
            hook_restraint_hint,
            confidence: 0.95,
            provenance_refs: vec!["test.phrase-audio-features".into()],
        }
    }
}
