use riotbox_core::{
    session::{
        Mc202RoleState, Mc202SourcePhraseNoteBudgetState, Mc202SourcePhrasePlanState,
        Mc202SourcePhraseSlotState, SessionFile,
    },
    source_graph::{
        AssetType, CandidateType, EnergyClass, Mc202SourcePhraseFeatureVector, PhraseSpan, Section,
        SectionLabelHint, SourceGraph, SourceTimingAnchorType, mc202_source_phrase_feature_vector,
        section_for_transport_bar,
    },
    transport::{CommitBoundaryState, TransportClockState},
};

mod candidate_families;

use candidate_families::choose_source_phrase_candidate;

pub(super) fn derive_mc202_source_phrase_plan(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
    boundary: Option<&CommitBoundaryState>,
    role: Mc202RoleState,
    touch: f32,
) -> Option<Mc202SourcePhrasePlanState> {
    let graph = source_graph?;

    let trusted_grid = session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .is_some_and(|confirmed| confirmed.source_id == graph.source.source_id);
    let boundary = boundary?;

    if !trusted_grid {
        return None;
    }

    let phrase_slot = source_phrase_slot_for_boundary(graph, boundary)?;

    let section = section_for_transport_bar(graph, &transport_clock_from_boundary(boundary));
    let features = mc202_source_phrase_feature_vector(graph, phrase_slot);
    let source_fallback_reason = source_phrase_fallback_reason(&features);
    let candidate_selection = choose_source_phrase_candidate(
        graph,
        role,
        section,
        phrase_slot,
        &features,
        session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref(),
    );
    let fallback_reason = source_fallback_reason.or(candidate_selection.fallback_reason.clone());
    let rhythm_cells =
        if fallback_reason.is_none() && candidate_selection.candidate_family.is_source_derived() {
            candidate_selection.rhythm_cells
        } else {
            [None; 16]
        };
    let confidence = source_phrase_confidence(graph, section, phrase_slot, &features);

    Some(Mc202SourcePhrasePlanState {
        source_id: graph.source.source_id.clone(),
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: phrase_slot.phrase_index,
            start_bar: phrase_slot.start_bar,
            end_bar: phrase_slot.end_bar,
        },
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
    })
}

fn source_phrase_slot_for_boundary<'a>(
    graph: &'a SourceGraph,
    boundary: &CommitBoundaryState,
) -> Option<&'a PhraseSpan> {
    let bar_index = boundary.bar_index as u32;
    graph
        .timing
        .phrase_grid
        .iter()
        .find(|phrase| bar_index >= phrase.start_bar && bar_index <= phrase.end_bar)
}

fn transport_clock_from_boundary(boundary: &CommitBoundaryState) -> TransportClockState {
    TransportClockState {
        is_playing: true,
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
    features: &Mc202SourcePhraseFeatureVector,
    fingerprint: Mc202SourcePhraseFingerprint,
) {
    if !fingerprint.strong_source || features.source_strength < 0.55 {
        return;
    }

    let index = feature_step(
        features.source_strength.max(features.transient_density),
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
    features: &Mc202SourcePhraseFeatureVector,
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
    let pressure_offset = if features.low_band_pressure > 0.72 {
        -5
    } else if features.offbeat_density > 0.55 {
        2
    } else {
        0
    };
    (section_offset + pressure_offset).clamp(-12, 12)
}

pub(super) fn source_phrase_note_budget(
    role: Mc202RoleState,
    section: Option<&Section>,
    features: &Mc202SourcePhraseFeatureVector,
) -> Mc202SourcePhraseNoteBudgetState {
    if features.hook_restraint > 0.72 || features.source_strength < 0.40 {
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

fn source_phrase_fallback_reason(features: &Mc202SourcePhraseFeatureVector) -> Option<String> {
    if features.stay_out {
        return Some("stay_out_source_context".into());
    }
    if !features.has_musical_evidence() {
        return Some("weak_source_phrase_features".into());
    }
    None
}
