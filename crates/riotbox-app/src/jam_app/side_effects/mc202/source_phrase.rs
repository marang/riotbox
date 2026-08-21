use riotbox_core::{
    session::{
        Mc202RoleState, Mc202SourcePhraseExpressionState, Mc202SourcePhraseNoteBudgetState,
        Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState, SessionFile,
    },
    source_graph::{
        AssetType, CandidateType, EnergyClass, Mc202SourcePhraseFeatureVector, PhraseSpan, Section,
        SectionLabelHint, SourceGraph, SourceTimingAnchorType, mc202_source_phrase_feature_vector,
        section_for_projected_scene, section_for_transport_bar,
    },
    transport::{CommitBoundaryState, TransportClockState},
};

mod candidate_families;

use candidate_families::choose_source_phrase_candidate;

struct Mc202PhraseMemoryRequest<'a> {
    previous_plan: Option<&'a Mc202SourcePhrasePlanState>,
    explicit_mutation: bool,
}

pub(super) fn derive_mc202_source_phrase_plan(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
    boundary: Option<&CommitBoundaryState>,
    role: Mc202RoleState,
    touch: f32,
    explicit_phrase_mutation: bool,
) -> Result<Option<Mc202SourcePhrasePlanState>, &'static str> {
    let Some(graph) = source_graph else {
        return Ok(None);
    };

    let trusted_grid = session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .is_some_and(|confirmed| confirmed.source_id == graph.source.source_id);
    let Some(boundary) = boundary else {
        return Ok(None);
    };

    if !trusted_grid {
        return Ok(None);
    }

    let Some(phrase_slot) = source_phrase_slot_for_boundary(graph, boundary) else {
        return Ok(None);
    };

    let section = source_section_for_boundary(graph, boundary);
    let features = mc202_source_phrase_feature_vector(graph, &phrase_slot);
    if let Some(feature_section_id) = features.source_section_id.as_ref() {
        match section {
            Some(boundary_section) if boundary_section.section_id == *feature_section_id => {}
            Some(_) => {
                return Err(
                    "phrase crosses source sections and feature ownership does not match the commit boundary",
                );
            }
            None => {
                return Err(
                    "source section ownership cannot be proven at the MC-202 commit boundary",
                );
            }
        }
    }
    let source_expression = mc202_source_phrase_expression_state(&features);
    let source_fallback_reason = source_phrase_fallback_reason(&features, &source_expression);
    let candidate_selection = choose_source_phrase_candidate(
        graph,
        role,
        section,
        &phrase_slot,
        &features,
        &source_expression,
        Mc202PhraseMemoryRequest {
            previous_plan: session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .as_ref(),
            explicit_mutation: explicit_phrase_mutation,
        },
    );
    let fallback_reason = source_fallback_reason.or(candidate_selection.fallback_reason.clone());
    let rhythm_cells =
        if fallback_reason.is_none() && candidate_selection.candidate_family.is_source_derived() {
            candidate_selection.rhythm_cells
        } else {
            [None; 16]
        };
    let confidence = source_phrase_confidence(graph, section, &phrase_slot, &features);

    Ok(Some(Mc202SourcePhrasePlanState {
        source_id: graph.source.source_id.clone(),
        source_section_id: features.source_section_id.clone(),
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: phrase_slot.phrase_index,
            start_bar: phrase_slot.start_bar,
            end_bar: phrase_slot.end_bar,
        },
        source_expression: Some(source_expression),
        role,
        rhythm_cells,
        note_budget: candidate_selection.note_budget,
        touch: touch.clamp(0.0, 1.0),
        confidence,
        candidate_family: Some(candidate_selection.candidate_family),
        candidate_count: candidate_selection.candidate_count,
        rejected_candidate_count: candidate_selection.rejected_candidate_count,
        candidate_provenance_refs: candidate_selection.provenance_refs,
        candidate_scorecards: candidate_selection.scorecards,
        phrase_memory_distance: candidate_selection.phrase_memory_distance,
        fallback_reason,
    }))
}

fn source_phrase_slot_for_boundary(
    graph: &SourceGraph,
    boundary: &CommitBoundaryState,
) -> Option<PhraseSpan> {
    let bar_index = boundary.bar_index as u32;
    let primary = graph.timing.primary_hypothesis();
    source_phrase_slot_for_projected_scene(graph, boundary)
        .or_else(|| {
            primary.and_then(|hypothesis| {
                hypothesis
                    .phrase_grid
                    .iter()
                    .find(|phrase| bar_index >= phrase.start_bar && bar_index <= phrase.end_bar)
                    .copied()
            })
        })
        .or_else(|| {
            graph
                .timing
                .phrase_grid
                .iter()
                .find(|phrase| bar_index >= phrase.start_bar && bar_index <= phrase.end_bar)
                .copied()
        })
        .or_else(|| {
            let hypothesis = primary?;
            let phrase_index = boundary.phrase_index.try_into().ok()?;
            let start_bar = hypothesis.bar_grid.first()?.bar_index;
            let end_bar = hypothesis.bar_grid.last()?.bar_index;
            (bar_index >= start_bar && bar_index <= end_bar).then_some(PhraseSpan {
                phrase_index,
                start_bar,
                end_bar,
                confidence: hypothesis.confidence,
            })
        })
}

fn source_phrase_slot_for_projected_scene(
    graph: &SourceGraph,
    boundary: &CommitBoundaryState,
) -> Option<PhraseSpan> {
    let section = boundary
        .scene_id
        .as_ref()
        .and_then(|scene_id| section_for_projected_scene(graph, scene_id))?;
    let primary = graph.timing.primary_hypothesis()?;

    if let Some(phrase) = primary
        .phrase_grid
        .iter()
        .find(|phrase| phrase.start_bar <= section.bar_end && phrase.end_bar >= section.bar_start)
        .copied()
    {
        return Some(PhraseSpan {
            phrase_index: phrase.phrase_index,
            start_bar: phrase.start_bar.max(section.bar_start),
            end_bar: phrase.end_bar.min(section.bar_end),
            confidence: phrase.confidence.min(section.confidence),
        });
    }

    let grid_start = primary.bar_grid.first()?.bar_index;
    let grid_end = primary.bar_grid.last()?.bar_index;
    let start_bar = section.bar_start.max(grid_start);
    let end_bar = section.bar_end.min(grid_end);
    (start_bar <= end_bar).then_some(PhraseSpan {
        phrase_index: boundary.phrase_index.try_into().ok()?,
        start_bar,
        end_bar,
        confidence: primary.confidence.min(section.confidence),
    })
}

fn source_section_for_boundary<'a>(
    graph: &'a SourceGraph,
    boundary: &CommitBoundaryState,
) -> Option<&'a Section> {
    boundary
        .scene_id
        .as_ref()
        .and_then(|scene_id| section_for_projected_scene(graph, scene_id))
        .or_else(|| section_for_transport_bar(graph, &transport_clock_from_boundary(boundary)))
}

fn transport_clock_from_boundary(boundary: &CommitBoundaryState) -> TransportClockState {
    TransportClockState {
        is_playing: true,
        // Session V1 commit boundaries persist this as the zero-based cursor.
        position_beats: boundary.beat_index as f64,
        beat_index: boundary.beat_index,
        bar_index: boundary.bar_index,
        phrase_index: boundary.phrase_index,
        current_scene: boundary.scene_id.clone(),
    }
}

pub(super) fn feature_step(feature: f32, tie_break: u8, offset: usize) -> usize {
    let bucket = (feature.clamp(0.0, 1.0) * 7.0).round() as usize;
    (offset + bucket + usize::from(tie_break % 4)) % 16
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Mc202SourcePhraseFingerprint {
    step_rotation: u8,
    accent_step: u8,
    interval_shift: i8,
    strong_source: bool,
}

pub(super) fn source_phrase_fingerprint(
    graph: &SourceGraph,
    section: Option<&Section>,
    phrase_slot: &PhraseSpan,
) -> Mc202SourcePhraseFingerprint {
    let seed = source_phrase_hash(graph, section, phrase_slot);
    let strong_source = graph.analysis_summary.hook_candidate_count > 0
        || graph.hook_candidate_count() > 0
        || graph.candidate_count(CandidateType::CaptureCandidate) > 0
        || graph
            .timing
            .primary_hypothesis()
            .is_some_and(|hypothesis| hypothesis.anchors.len() >= 4);

    Mc202SourcePhraseFingerprint {
        step_rotation: ((seed % 4) * 2) as u8,
        accent_step: ((seed >> 5) % 16) as u8,
        interval_shift: match (seed >> 9) % 5 {
            0 => -2,
            1 => -1,
            2 => 0,
            3 => 1,
            _ => 2,
        },
        strong_source,
    }
}

fn source_phrase_hash(
    graph: &SourceGraph,
    section: Option<&Section>,
    phrase_slot: &PhraseSpan,
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    hash_str(&mut hash, &graph.source.content_hash);
    hash_str(&mut hash, &graph.provenance.source_hash);
    hash_u64(&mut hash, graph.provenance.analysis_seed);
    hash_u64(
        &mut hash,
        graph
            .timing
            .bpm_estimate
            .map_or(0, |bpm| (bpm * 100.0).round() as u64),
    );
    hash_u64(&mut hash, phrase_slot.phrase_index as u64);
    hash_u64(&mut hash, phrase_slot.start_bar as u64);
    hash_u64(
        &mut hash,
        graph.analysis_summary.hook_candidate_count as u64,
    );
    hash_u64(
        &mut hash,
        graph.analysis_summary.loop_candidate_count as u64,
    );

    if let Some(section) = section {
        hash_u64(&mut hash, section.bar_start as u64);
        hash_u64(&mut hash, section.bar_end as u64);
        hash_u64(
            &mut hash,
            source_phrase_section_label_code(section.label_hint),
        );
        hash_u64(&mut hash, source_phrase_energy_code(section.energy_class));
        for tag in &section.tags {
            hash_str(&mut hash, tag);
        }
    }

    for asset in graph.assets.iter().take(8) {
        hash_u64(&mut hash, source_phrase_asset_type_code(asset.asset_type));
        hash_u64(&mut hash, asset.start_bar as u64);
        hash_u64(&mut hash, asset.end_bar as u64);
        hash_u64(&mut hash, (asset.confidence * 1000.0).round() as u64);
        for tag in asset.tags.iter().take(4) {
            hash_str(&mut hash, tag);
        }
    }

    for candidate in graph.candidates.iter().take(8) {
        hash_u64(
            &mut hash,
            source_phrase_candidate_type_code(candidate.candidate_type),
        );
        hash_u64(&mut hash, (candidate.score * 1000.0).round() as u64);
        hash_u64(&mut hash, (candidate.confidence * 1000.0).round() as u64);
        for tag in candidate.tags.iter().take(4) {
            hash_str(&mut hash, tag);
        }
    }

    if let Some(hypothesis) = graph.timing.primary_hypothesis() {
        hash_u64(&mut hash, hypothesis.anchors.len() as u64);
        for anchor in hypothesis.anchors.iter().take(12) {
            hash_u64(
                &mut hash,
                source_phrase_anchor_type_code(anchor.anchor_type),
            );
            hash_u64(&mut hash, anchor.beat_index.unwrap_or_default() as u64);
            hash_u64(&mut hash, (anchor.strength * 1000.0).round() as u64);
        }
    }

    hash
}

fn hash_str(hash: &mut u64, value: &str) {
    for byte in value.as_bytes() {
        hash_u64(hash, u64::from(*byte));
    }
}

fn hash_u64(hash: &mut u64, value: u64) {
    *hash ^= value;
    *hash = hash.wrapping_mul(0x100000001b3);
}

pub(super) fn add_source_phrase_accent(
    role: Mc202RoleState,
    cells: &mut [Option<i8>; 16],
    expression: &Mc202SourcePhraseExpressionState,
    fingerprint: Mc202SourcePhraseFingerprint,
) {
    if !fingerprint.strong_source
        || expression.phrase_density < 0.38
        || expression.confidence < 0.45
    {
        return;
    }

    let index = feature_step(
        expression
            .phrase_density
            .max(expression.transient_backbeat)
            .max(expression.bass_pressure),
        fingerprint.accent_step,
        1,
    );
    if cells[index].is_some() {
        return;
    }

    let accent = match role {
        Mc202RoleState::Pressure => -12,
        Mc202RoleState::Answer => 7,
        Mc202RoleState::Instigator => 19,
        Mc202RoleState::Leader | Mc202RoleState::Follower => 0,
    };
    cells[index] = Some((accent + fingerprint.interval_shift).clamp(-24, 24));
}

fn source_phrase_section_label_code(label: SectionLabelHint) -> u64 {
    match label {
        SectionLabelHint::Intro => 1,
        SectionLabelHint::Build => 2,
        SectionLabelHint::Drop => 3,
        SectionLabelHint::Break => 4,
        SectionLabelHint::Verse => 5,
        SectionLabelHint::Chorus => 6,
        SectionLabelHint::Bridge => 7,
        SectionLabelHint::Outro => 8,
        SectionLabelHint::Unknown => 0,
    }
}

fn source_phrase_energy_code(energy: EnergyClass) -> u64 {
    match energy {
        EnergyClass::Low => 1,
        EnergyClass::Medium => 2,
        EnergyClass::High => 3,
        EnergyClass::Peak => 4,
        EnergyClass::Unknown => 0,
    }
}

fn source_phrase_asset_type_code(asset_type: AssetType) -> u64 {
    match asset_type {
        AssetType::Slice => 1,
        AssetType::LoopWindow => 2,
        AssetType::HookFragment => 3,
        AssetType::DrumAnchor => 4,
        AssetType::PhraseFragment => 5,
        AssetType::TextureFragment => 6,
    }
}

fn source_phrase_candidate_type_code(candidate_type: CandidateType) -> u64 {
    match candidate_type {
        CandidateType::KickAnchor => 1,
        CandidateType::SnareAnchor => 2,
        CandidateType::GhostHit => 3,
        CandidateType::FillFragment => 4,
        CandidateType::LoopCandidate => 5,
        CandidateType::HookCandidate => 6,
        CandidateType::AnswerCandidate => 7,
        CandidateType::CaptureCandidate => 8,
    }
}

fn source_phrase_anchor_type_code(anchor_type: SourceTimingAnchorType) -> u64 {
    match anchor_type {
        SourceTimingAnchorType::Kick => 1,
        SourceTimingAnchorType::Snare => 2,
        SourceTimingAnchorType::Backbeat => 3,
        SourceTimingAnchorType::Fill => 4,
        SourceTimingAnchorType::LoopWindow => 5,
        SourceTimingAnchorType::AnswerSlot => 6,
        SourceTimingAnchorType::CaptureCandidate => 7,
        SourceTimingAnchorType::TransientCluster => 8,
    }
}

pub(super) fn source_phrase_contour_offset(
    section: Option<&Section>,
    expression: &Mc202SourcePhraseExpressionState,
) -> i8 {
    let section_offset = match section.map(|section| (section.label_hint, section.energy_class)) {
        Some((SectionLabelHint::Build, _)) => 2,
        Some((
            SectionLabelHint::Drop | SectionLabelHint::Chorus,
            EnergyClass::High | EnergyClass::Peak,
        )) => -2,
        Some((SectionLabelHint::Break | SectionLabelHint::Intro | SectionLabelHint::Outro, _)) => {
            -5
        }
        Some((_, EnergyClass::Low)) => -5,
        _ => 0,
    };
    let pressure_offset = if expression.bass_pressure > 0.72 {
        -5
    } else if expression.offbeat_answer_space > 0.55 {
        2
    } else if expression.stab_bite > 0.62 {
        3
    } else {
        0
    };
    (section_offset + pressure_offset).clamp(-12, 12)
}

pub(super) fn source_phrase_note_budget(
    role: Mc202RoleState,
    section: Option<&Section>,
    expression: &Mc202SourcePhraseExpressionState,
) -> Mc202SourcePhraseNoteBudgetState {
    if expression.stay_out_pressure > 0.72
        || expression.hook_restraint > 0.72
        || expression.phrase_density < 0.28
    {
        return Mc202SourcePhraseNoteBudgetState::Sparse;
    }

    match role {
        Mc202RoleState::Pressure | Mc202RoleState::Answer => {
            Mc202SourcePhraseNoteBudgetState::Sparse
        }
        Mc202RoleState::Instigator => Mc202SourcePhraseNoteBudgetState::Push,
        Mc202RoleState::Leader | Mc202RoleState::Follower
            if section.is_some_and(|section| {
                matches!(section.energy_class, EnergyClass::High | EnergyClass::Peak)
            }) =>
        {
            Mc202SourcePhraseNoteBudgetState::Wide
        }
        Mc202RoleState::Leader | Mc202RoleState::Follower => {
            Mc202SourcePhraseNoteBudgetState::Balanced
        }
    }
}

fn source_phrase_confidence(
    graph: &SourceGraph,
    section: Option<&Section>,
    phrase_slot: &PhraseSpan,
    features: &Mc202SourcePhraseFeatureVector,
) -> f32 {
    let timing = graph.timing.bpm_confidence.max(phrase_slot.confidence);
    let section_confidence = section.map_or(0.5, |section| section.confidence);
    ((timing + section_confidence + phrase_slot.confidence + features.confidence) / 4.0)
        .clamp(0.0, 1.0)
}

fn source_phrase_fallback_reason(
    features: &Mc202SourcePhraseFeatureVector,
    expression: &Mc202SourcePhraseExpressionState,
) -> Option<String> {
    if features.stay_out || expression.stay_out_pressure >= 0.90 {
        return Some("stay_out_source_context".into());
    }
    if !features.has_musical_evidence() || expression.confidence < 0.35 {
        return Some("weak_source_phrase_features".into());
    }
    None
}

fn mc202_source_phrase_expression_state(
    features: &Mc202SourcePhraseFeatureVector,
) -> Mc202SourcePhraseExpressionState {
    let bass_pressure =
        (features.low_band_pressure * 0.74 + features.low_band_movement * 0.26).clamp(0.0, 1.0);
    let transient_backbeat =
        (features.backbeat_density * 0.58 + features.transient_density * 0.42).clamp(0.0, 1.0);
    let offbeat_answer_space = (features.offbeat_density * (1.0 - features.hook_restraint * 0.28)
        + features.transient_density * 0.12)
        .clamp(0.0, 1.0);
    let phrase_density = (features.transient_density * 0.44
        + features.offbeat_density * 0.24
        + features.source_strength * 0.24
        + features.backbeat_density * 0.08)
        .clamp(0.0, 1.0);
    let stab_bite = (features.spectral_roughness * 0.36
        + features.spectral_brightness * 0.24
        + features.transient_density * 0.25
        + features.backbeat_density * 0.15)
        .clamp(0.0, 1.0);
    let stay_out_pressure = if features.stay_out {
        1.0
    } else {
        (features.hook_restraint * 0.42
            + (1.0 - features.source_strength) * 0.34
            + (1.0 - features.confidence) * 0.24)
            .clamp(0.0, 1.0)
    };

    let mut provenance_refs = vec![
        format!(
            "expression:low_pressure_contour:{:.3}",
            features.low_band_movement
        ),
        format!("expression:bass_pressure:{bass_pressure:.3}"),
        format!("expression:transient_backbeat:{transient_backbeat:.3}"),
        format!("expression:offbeat_answer_space:{offbeat_answer_space:.3}"),
        format!("expression:phrase_density:{phrase_density:.3}"),
        format!("expression:hook_restraint:{:.3}", features.hook_restraint),
        format!("expression:stab_bite:{stab_bite:.3}"),
    ];
    provenance_refs.extend(
        features
            .provenance_refs
            .iter()
            .take(10)
            .map(|reference| format!("source_expression:{reference}")),
    );

    Mc202SourcePhraseExpressionState {
        low_pressure_contour: features.low_band_movement.clamp(0.0, 1.0),
        bass_pressure,
        transient_backbeat,
        offbeat_answer_space,
        phrase_density,
        hook_restraint: features.hook_restraint.clamp(0.0, 1.0),
        stab_bite,
        stay_out_pressure,
        confidence: features.confidence.clamp(0.0, 1.0),
        provenance_refs,
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        ids::{SceneId, SectionId, SourceId},
        source_graph::{
            BarSpan, DecodeProfile, EnergyClass, GraphProvenance, MeterHint, PhraseSpan, Section,
            SectionLabelHint, SourceDescriptor, SourceGraph, TimingHypothesis,
            TimingHypothesisKind, TimingQuality,
        },
        transport::CommitBoundaryState,
    };

    use super::{source_phrase_slot_for_boundary, source_section_for_boundary};

    #[test]
    fn phrase_slot_prefers_selected_primary_grid_over_divergent_top_level_grid() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("primary-phrase-test"),
                path: "primary-phrase.wav".into(),
                content_hash: "primary-phrase-hash".into(),
                duration_seconds: 16.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec!["test".into()],
                generated_at: "2026-07-16T00:00:00Z".into(),
                source_hash: "primary-phrase-hash".into(),
                analysis_seed: 23,
                run_notes: None,
            },
        );
        graph.timing.phrase_grid = vec![PhraseSpan {
            phrase_index: 2,
            start_bar: 5,
            end_bar: 8,
            confidence: 0.5,
        }];
        graph.timing.primary_hypothesis_id = Some("selected-primary".into());
        graph.timing.hypotheses = vec![TimingHypothesis {
            hypothesis_id: "selected-primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 132.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.94,
            score: 0.94,
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: vec![PhraseSpan {
                phrase_index: 9,
                start_bar: 5,
                end_bar: 8,
                confidence: 0.94,
            }],
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: vec!["test:selected-primary".into()],
        }];
        let boundary = CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 19,
            bar_index: 5,
            phrase_index: 2,
            scene_id: None,
        };

        let slot = source_phrase_slot_for_boundary(&graph, &boundary).expect("primary phrase slot");

        assert_eq!(slot.phrase_index, 9);
        assert_eq!((slot.start_bar, slot.end_bar), (5, 8));
    }

    #[test]
    fn short_source_projects_later_performance_boundary_into_scene_section() {
        let mut graph = short_source_graph();
        graph.timing.hypotheses[0].phrase_grid.clear();
        let boundary = projected_boundary("scene-01-intro");

        let section =
            source_section_for_boundary(&graph, &boundary).expect("projected source section");
        let slot =
            source_phrase_slot_for_boundary(&graph, &boundary).expect("projected phrase slot");

        assert_eq!(section.section_id, SectionId::from("section-intro"));
        assert_eq!(slot.phrase_index, 2);
        assert_eq!((slot.start_bar, slot.end_bar), (1, 1));
        assert_eq!(slot.confidence, 0.88);
    }

    #[test]
    fn short_source_uses_source_phrase_grid_owned_by_projected_scene() {
        let graph = short_source_graph();
        let boundary = projected_boundary("scene-01-intro");

        let slot =
            source_phrase_slot_for_boundary(&graph, &boundary).expect("source phrase grid slot");

        assert_eq!(slot.phrase_index, 7);
        assert_eq!((slot.start_bar, slot.end_bar), (1, 1));
        assert_eq!(slot.confidence, 0.88);
    }

    #[test]
    fn projected_phrase_is_clamped_to_its_scene_owned_source_section() {
        let graph = short_source_graph();
        let boundary = projected_boundary("scene-02-drop");

        let section =
            source_section_for_boundary(&graph, &boundary).expect("projected drop section");
        let slot =
            source_phrase_slot_for_boundary(&graph, &boundary).expect("projected drop phrase");

        assert_eq!(section.section_id, SectionId::from("section-drop"));
        assert_eq!(slot.phrase_index, 7);
        assert_eq!((slot.start_bar, slot.end_bar), (2, 2));
        assert_eq!(slot.confidence, 0.79);
    }

    #[test]
    fn projected_scene_ownership_wins_over_transport_bar_phrase_overlap() {
        let graph = short_source_graph();
        let mut boundary = projected_boundary("scene-01-intro");
        boundary.bar_index = 2;

        let slot = source_phrase_slot_for_boundary(&graph, &boundary).expect("scene-owned phrase");

        assert_eq!((slot.start_bar, slot.end_bar), (1, 1));
    }

    #[test]
    fn short_source_without_known_projected_scene_stays_unavailable() {
        let graph = short_source_graph();
        let boundary = projected_boundary("scene-03-unknown");

        assert!(source_section_for_boundary(&graph, &boundary).is_none());
        assert!(source_phrase_slot_for_boundary(&graph, &boundary).is_none());
    }

    fn short_source_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("short-source"),
                path: "short-source.wav".into(),
                content_hash: "short-source-hash".into(),
                duration_seconds: 3.7,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec!["test".into()],
                generated_at: "2026-07-17T00:00:00Z".into(),
                source_hash: "short-source-hash".into(),
                analysis_seed: 24,
                run_notes: None,
            },
        );
        graph.sections = vec![
            Section {
                section_id: SectionId::from("section-intro"),
                label_hint: SectionLabelHint::Intro,
                start_seconds: 0.0,
                end_seconds: 1.85,
                bar_start: 1,
                bar_end: 1,
                energy_class: EnergyClass::Medium,
                confidence: 0.88,
                tags: vec!["short-loop".into()],
            },
            Section {
                section_id: SectionId::from("section-drop"),
                label_hint: SectionLabelHint::Drop,
                start_seconds: 1.85,
                end_seconds: 3.7,
                bar_start: 2,
                bar_end: 2,
                energy_class: EnergyClass::High,
                confidence: 0.79,
                tags: Vec::new(),
            },
        ];
        graph.timing.primary_hypothesis_id = Some("short-primary".into());
        graph.timing.hypotheses = vec![TimingHypothesis {
            hypothesis_id: "short-primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 130.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.92,
            score: 0.92,
            beat_grid: Vec::new(),
            bar_grid: vec![
                BarSpan {
                    bar_index: 1,
                    start_seconds: 0.0,
                    end_seconds: 1.85,
                    downbeat_confidence: 0.91,
                    phrase_index: Some(7),
                },
                BarSpan {
                    bar_index: 2,
                    start_seconds: 1.85,
                    end_seconds: 3.7,
                    downbeat_confidence: 0.9,
                    phrase_index: Some(7),
                },
            ],
            phrase_grid: vec![PhraseSpan {
                phrase_index: 7,
                start_bar: 1,
                end_bar: 2,
                confidence: 0.91,
            }],
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: vec!["test:short-primary".into()],
        }];
        graph
    }

    fn projected_boundary(scene_id: &str) -> CommitBoundaryState {
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 19,
            bar_index: 5,
            phrase_index: 2,
            scene_id: Some(SceneId::from(scene_id)),
        }
    }
}
