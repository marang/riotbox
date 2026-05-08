use super::*;

#[test]
fn source_timing_probe_bpm_candidates_classify_primary_grid_anchors() {
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "classified-anchors-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
            &[1.0, 0.7, 0.35, 0.8, 1.1, 0.75, 0.3, 0.85],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    let anchors = primary
        .anchors
        .iter()
        .map(|anchor| {
            (
                anchor.anchor_type,
                anchor.bar_index,
                anchor.beat_index,
                anchor.tags.clone(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        anchors
            .iter()
            .filter(|(anchor_type, _, _, _)| *anchor_type == SourceTimingAnchorType::Kick)
            .count(),
        2
    );
    assert_eq!(
        anchors
            .iter()
            .filter(|(anchor_type, _, _, _)| *anchor_type == SourceTimingAnchorType::Backbeat)
            .count(),
        4
    );
    assert!(anchors.iter().any(|(anchor_type, bar, beat, tags)| {
        *anchor_type == SourceTimingAnchorType::Kick
            && *bar == Some(1)
            && *beat == Some(1)
            && tags.contains(&"kick_anchor".into())
            && tags.contains(&"downbeat".into())
            && tags.contains(&"grid_aligned".into())
    }));
    assert!(anchors.iter().any(|(anchor_type, bar, beat, tags)| {
        *anchor_type == SourceTimingAnchorType::Backbeat
            && *bar == Some(1)
            && *beat == Some(2)
            && tags.contains(&"backbeat_anchor".into())
            && tags.contains(&"snare_style".into())
            && tags.contains(&"beat_in_bar_2".into())
    }));
    assert!(anchors.iter().any(|(anchor_type, bar, beat, tags)| {
        *anchor_type == SourceTimingAnchorType::TransientCluster
            && *bar == Some(1)
            && *beat == Some(3)
            && tags.contains(&"transient_cluster".into())
            && tags.contains(&"beat_in_bar_3".into())
    }));
}

#[test]
fn source_timing_probe_bpm_candidates_keep_anchor_classes_generic_when_downbeat_is_ambiguous() {
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "ambiguous-anchor-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(primary
        .anchors
        .iter()
        .all(|anchor| anchor.anchor_type == SourceTimingAnchorType::TransientCluster));
    assert!(primary
        .anchors
        .iter()
        .all(|anchor| anchor.tags.contains(&"anchor_classified_v0".into())));
}

#[test]
fn source_timing_probe_bpm_candidates_keep_alternate_hypothesis_anchors_generic() {
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "alternate-anchors-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
            &[1.0, 0.7, 0.35, 0.8, 1.1, 0.75, 0.3, 0.85],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert!(timing
        .primary_hypothesis()
        .expect("primary hypothesis")
        .anchors
        .iter()
        .any(|anchor| anchor.anchor_type == SourceTimingAnchorType::Kick));
    assert!(timing
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.kind != TimingHypothesisKind::Primary)
        .flat_map(|hypothesis| hypothesis.anchors.iter())
        .all(|anchor| anchor.anchor_type == SourceTimingAnchorType::TransientCluster));
}
