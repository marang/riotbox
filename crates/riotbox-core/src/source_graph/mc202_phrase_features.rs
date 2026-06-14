#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mc202SourcePhraseFeatureVector {
    pub phrase_index: u32,
    pub low_band_pressure: f32,
    pub transient_density: f32,
    pub offbeat_density: f32,
    pub hook_restraint: f32,
    pub source_strength: f32,
    pub stay_out: bool,
    pub confidence: Confidence,
    pub provenance_refs: Vec<String>,
}

impl Mc202SourcePhraseFeatureVector {
    #[must_use]
    pub fn has_musical_evidence(&self) -> bool {
        !self.stay_out && self.source_strength >= 0.35 && self.confidence >= 0.35
    }
}

#[must_use]
pub fn mc202_source_phrase_feature_vector(
    graph: &SourceGraph,
    phrase: &PhraseSpan,
) -> Mc202SourcePhraseFeatureVector {
    let section = graph.sections.iter().find(|section| {
        phrase_bar_ranges_overlap(
            phrase.start_bar,
            phrase.end_bar,
            section.bar_start,
            section.bar_end,
        )
    });
    let phrase_assets = graph
        .assets
        .iter()
        .filter(|asset| {
            phrase_bar_ranges_overlap(
                phrase.start_bar,
                phrase.end_bar,
                asset.start_bar,
                asset.end_bar,
            )
        })
        .collect::<Vec<_>>();
    let phrase_candidates = graph
        .candidates
        .iter()
        .filter(|candidate| {
            phrase_assets
                .iter()
                .any(|asset| asset.asset_id == candidate.asset_ref)
        })
        .collect::<Vec<_>>();
    let anchors = graph
        .timing
        .primary_hypothesis()
        .map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());
    let phrase_anchors = anchors
        .iter()
        .filter(|anchor| {
            anchor
                .bar_index
                .is_some_and(|bar| bar >= phrase.start_bar && bar <= phrase.end_bar)
        })
        .collect::<Vec<_>>();
    let measured_audio = phrase_audio_features_for_phrase(graph, phrase);

    let mut low_band_pressure = clamp01(
        section.map_or(0.0, |section| match section.energy_class {
            EnergyClass::Low => 0.18,
            EnergyClass::Medium => 0.42,
            EnergyClass::High => 0.70,
            EnergyClass::Peak => 0.88,
            EnergyClass::Unknown => 0.0,
        }) + average_anchor_strength(&phrase_anchors, &[SourceTimingAnchorType::Kick]) * 0.35
            + asset_type_presence(&phrase_assets, AssetType::DrumAnchor) * 0.22
            + candidate_type_presence(&phrase_candidates, CandidateType::KickAnchor) * 0.18
            + tag_presence_in_phrase(
                section,
                &phrase_assets,
                &phrase_candidates,
                &["bass", "low", "pressure"],
            ) * 0.15,
    );
    let mut transient_density = clamp01(
        weighted_anchor_presence(
            &phrase_anchors,
            &[
                SourceTimingAnchorType::Kick,
                SourceTimingAnchorType::Snare,
                SourceTimingAnchorType::Backbeat,
                SourceTimingAnchorType::Fill,
                SourceTimingAnchorType::TransientCluster,
            ],
        ) + asset_type_presence(&phrase_assets, AssetType::DrumAnchor) * 0.20
            + candidate_type_presence(&phrase_candidates, CandidateType::GhostHit) * 0.12
            + candidate_type_presence(&phrase_candidates, CandidateType::FillFragment) * 0.16,
    );
    let mut offbeat_density = clamp01(
        offbeat_anchor_presence(&phrase_anchors, graph.timing.meter_hint) * 0.75
            + candidate_type_presence(&phrase_candidates, CandidateType::AnswerCandidate) * 0.18
            + tag_presence_in_phrase(
                section,
                &phrase_assets,
                &phrase_candidates,
                &["offbeat", "syncopated"],
            ) * 0.18,
    );
    let mut hook_restraint = clamp01(
        section.map_or(0.0, |section| match section.label_hint {
            SectionLabelHint::Chorus => 0.75,
            SectionLabelHint::Drop => 0.45,
            SectionLabelHint::Build => 0.35,
            SectionLabelHint::Break => 0.20,
            SectionLabelHint::Intro | SectionLabelHint::Outro => 0.10,
            SectionLabelHint::Verse | SectionLabelHint::Bridge | SectionLabelHint::Unknown => 0.25,
        }) + asset_type_presence(&phrase_assets, AssetType::HookFragment) * 0.28
            + candidate_type_presence(&phrase_candidates, CandidateType::HookCandidate) * 0.30
            + (graph.analysis_summary.hook_candidate_count as f32).min(2.0) * 0.08
            + tag_presence_in_phrase(
                section,
                &phrase_assets,
                &phrase_candidates,
                &["hook", "vocal", "lead"],
            ) * 0.16,
    );
    let mut source_strength = clamp01(
        low_band_pressure * 0.38
            + transient_density * 0.30
            + offbeat_density * 0.12
            + graph.analysis_summary.overall_confidence.clamp(0.0, 1.0) * 0.10
            + phrase.confidence.clamp(0.0, 1.0) * 0.10,
    );
    let mut confidence = clamp01(
        phrase.confidence * 0.35
            + graph.timing.bpm_confidence * 0.25
            + section.map_or(0.35, |section| section.confidence) * 0.20
            + source_strength * 0.20,
    );
    if let Some(audio) = measured_audio.filter(|audio| audio.has_measured_evidence()) {
        low_band_pressure = clamp01(
            audio.low_band_rms * 2.2
                + audio.low_mid_ratio * 0.22
                + audio.low_band_movement * 0.34,
        );
        transient_density = clamp01(audio.transient_density);
        offbeat_density = clamp01(audio.offbeat_onset_density);
        hook_restraint = clamp01(
            audio.hook_restraint_hint * 0.70
                + audio.spectral_brightness * 0.12
                + audio.spectral_roughness * 0.08
                + hook_restraint * 0.10,
        );
        source_strength = clamp01(
            low_band_pressure * 0.42
                + transient_density * 0.24
                + offbeat_density * 0.12
                + audio.low_band_movement * 0.10
                + audio.confidence * 0.12,
        );
        confidence = clamp01(
            audio.confidence * 0.38
                + phrase.confidence * 0.24
                + graph.timing.bpm_confidence * 0.18
                + source_strength * 0.20,
        );
    }
    let stay_out = confidence < 0.35
        || source_strength < 0.25
        || (hook_restraint >= 0.82 && low_band_pressure < 0.45 && transient_density < 0.45);

    let mut provenance_refs =
        mc202_feature_provenance_refs(section, &phrase_assets, &phrase_candidates, &phrase_anchors);
    if let Some(audio) = measured_audio {
        let prefix = if audio.has_measured_evidence() {
            "phrase_audio"
        } else {
            "phrase_audio_untrusted"
        };
        provenance_refs.extend(
            audio
                .provenance_refs
                .iter()
                .map(|reference| format!("{prefix}:{reference}")),
        );
    }

    Mc202SourcePhraseFeatureVector {
        phrase_index: phrase.phrase_index,
        low_band_pressure,
        transient_density,
        offbeat_density,
        hook_restraint,
        source_strength,
        stay_out,
        confidence,
        provenance_refs,
    }
}

fn phrase_audio_features_for_phrase<'a>(
    graph: &'a SourceGraph,
    phrase: &PhraseSpan,
) -> Option<&'a PhraseAudioFeatures> {
    graph
        .phrase_audio_features
        .iter()
        .find(|features| features.phrase_index == phrase.phrase_index)
        .or_else(|| {
            graph.phrase_audio_features.iter().find(|features| {
                phrase_bar_ranges_overlap(
                    phrase.start_bar,
                    phrase.end_bar,
                    features.start_bar,
                    features.end_bar,
                )
            })
        })
}

fn phrase_bar_ranges_overlap(left_start: u32, left_end: u32, right_start: u32, right_end: u32) -> bool {
    left_start <= right_end && right_start <= left_end
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn average_anchor_strength(
    anchors: &[&SourceTimingAnchor],
    anchor_types: &[SourceTimingAnchorType],
) -> f32 {
    let mut count = 0_u32;
    let mut total = 0.0_f32;
    for anchor in anchors {
        if anchor_types.contains(&anchor.anchor_type) {
            count += 1;
            total += anchor.strength.clamp(0.0, 1.0) * anchor.confidence.clamp(0.0, 1.0);
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn weighted_anchor_presence(
    anchors: &[&SourceTimingAnchor],
    anchor_types: &[SourceTimingAnchorType],
) -> f32 {
    clamp01(average_anchor_strength(anchors, anchor_types) + (anchors.len() as f32 * 0.04))
}

fn offbeat_anchor_presence(anchors: &[&SourceTimingAnchor], meter_hint: Option<MeterHint>) -> f32 {
    let beats_per_bar = u32::from(meter_hint.map_or(4, |meter| meter.beats_per_bar.max(1)));
    let mut count = 0_u32;
    let mut total = 0.0_f32;
    for anchor in anchors {
        let Some(beat_index) = anchor.beat_index else {
            continue;
        };
        let beat_in_bar = beat_index % beats_per_bar;
        if beat_in_bar != 0 && beat_in_bar != beats_per_bar / 2 {
            count += 1;
            total += anchor.strength.clamp(0.0, 1.0) * anchor.confidence.clamp(0.0, 1.0);
        }
    }
    if count == 0 {
        0.0
    } else {
        clamp01(total / count as f32 + count as f32 * 0.05)
    }
}

fn asset_type_presence(assets: &[&Asset], asset_type: AssetType) -> f32 {
    if assets.iter().any(|asset| asset.asset_type == asset_type) {
        1.0
    } else {
        0.0
    }
}

fn candidate_type_presence(candidates: &[&Candidate], candidate_type: CandidateType) -> f32 {
    candidates
        .iter()
        .filter(|candidate| candidate.candidate_type == candidate_type)
        .map(|candidate| (candidate.score * candidate.confidence).clamp(0.0, 1.0))
        .fold(0.0, f32::max)
}

fn tag_presence_in_phrase(
    section: Option<&Section>,
    assets: &[&Asset],
    candidates: &[&Candidate],
    wanted_tags: &[&str],
) -> f32 {
    let mut best: f32 = 0.0;
    if section.is_some_and(|section| tags_contain_any(&section.tags, wanted_tags)) {
        best = best.max(1.0);
    }
    for asset in assets {
        if tags_contain_any(&asset.tags, wanted_tags) {
            best = best.max(asset.confidence.clamp(0.0, 1.0));
        }
    }
    for candidate in candidates {
        if tags_contain_any(&candidate.tags, wanted_tags) {
            best = best.max((candidate.score * candidate.confidence).clamp(0.0, 1.0));
        }
    }
    best
}

fn tags_contain_any(tags: &[String], wanted_tags: &[&str]) -> bool {
    tags.iter()
        .any(|tag| wanted_tags.iter().any(|wanted| tag.contains(wanted)))
}

fn mc202_feature_provenance_refs(
    section: Option<&Section>,
    assets: &[&Asset],
    candidates: &[&Candidate],
    anchors: &[&SourceTimingAnchor],
) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(section) = section {
        refs.push(format!("section:{}", section.section_id.as_str()));
    }
    refs.extend(
        assets
            .iter()
            .take(4)
            .map(|asset| format!("asset:{}", asset.asset_id.as_str())),
    );
    refs.extend(
        candidates
            .iter()
            .take(4)
            .map(|candidate| format!("candidate:{}", candidate.candidate_id.as_str())),
    );
    refs.extend(
        anchors
            .iter()
            .take(4)
            .map(|anchor| format!("anchor:{}", anchor.anchor_id)),
    );
    refs
}
