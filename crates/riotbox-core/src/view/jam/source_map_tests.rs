use crate::{
    action::CaptureLengthIntent,
    ids::{ActionId, SectionId, SourceId},
    queue::ActionQueue,
    session::{SessionFile, SourceTimingGridConfirmationState},
    source_graph::{
        BarSpan, DecodeProfile, EnergyClass, GraphProvenance, MeterHint, Section,
        SectionLabelHint, SourceDescriptor, SourceGraph, SourceMapBucket, SourceMapPeakClass,
        TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality,
    },
};

use super::*;

#[test]
fn source_map_uses_bar_grid_when_locked_bar_spans_exist() {
    let graph = source_map_test_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.transport.position_beats = 4.0;

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(vm.source.source_map.mode, SourceMapModeView::BarGrid);
    assert_eq!(vm.source.source_map.trust_label, "grid locked");
    assert_eq!(vm.source.source_map.width, 32);
    assert_eq!(vm.source.source_map.energy_row.chars().count(), 32);
    assert!(vm.source.source_map.energy_row.contains('█'));
    assert!(vm.source.source_map.peak_row.contains('█'));
    assert!(vm.source.source_map.grid_row.contains('|'));
    assert_eq!(vm.source.source_map.playhead_column, Some(8));
    assert!(vm.source.source_map.playhead_row.contains('^'));
    assert_eq!(
        vm.source.source_map.capture_range_row.chars().nth(16),
        Some('[')
    );
    assert_eq!(
        vm.source.source_map.capture_range_row.chars().nth(31),
        Some(']')
    );
    assert_eq!(
        vm.source.source_map.current_region_label,
        "now bar 2 | section A"
    );
    assert_eq!(
        vm.source.source_map.navigation_hint,
        "nav Left/Right bar | Up/Down phrase"
    );
    assert_eq!(
        vm.source.source_map.capture_hint,
        "cap next bar | map bar grid | 32 cols"
    );
    assert_eq!(
        vm.source.source_map.section_labels,
        vec!["section A 1-2", "drop 3-4"]
    );
}

#[test]
fn source_map_falls_back_to_time_when_grid_needs_confirmation() {
    let graph = source_map_test_graph(TimingDegradedPolicy::ManualConfirm, TimingQuality::Low);
    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(vm.source.source_map.mode, SourceMapModeView::TimeFallback);
    assert_eq!(vm.source.source_map.trust_label, "needs confirm");
    assert_eq!(vm.source.source_map.grid_row, ".".repeat(32));
    assert_eq!(vm.source.source_map.capture_range_row, ".".repeat(32));
    assert_eq!(
        vm.source.source_map.capture_hint,
        "cap listen first | map time fallback | no bar-accurate claim"
    );
}

#[test]
fn source_map_uses_confirmed_grid_without_mutating_analysis_cue() {
    let graph = source_map_test_graph(TimingDegradedPolicy::ManualConfirm, TimingQuality::Low);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph.source.source_id.clone(),
            hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
            confirmed_by_action: ActionId(42),
            confirmed_at: 123,
        });

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(SourceTimingSummaryView::from_graph(&graph).cue, "needs confirm");
    assert_eq!(vm.source.source_map.mode, SourceMapModeView::BarGrid);
    assert_eq!(vm.source.source_map.trust_label, "grid confirmed");
    assert!(vm.source.source_map.grid_row.contains('|'));
    assert!(vm.source.source_map.capture_range_row.contains('['));
    assert_eq!(
        vm.source.source_map.navigation_hint,
        "nav Left/Right bar | Up/Down phrase"
    );
    assert_eq!(
        vm.source.source_map.capture_hint,
        "cap next bar | map bar grid | 32 cols"
    );
}

#[test]
fn source_map_ignores_confirmation_for_different_hypothesis() {
    let graph = source_map_test_graph(TimingDegradedPolicy::ManualConfirm, TimingQuality::Low);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph.source.source_id.clone(),
            hypothesis_id: Some("alternate".into()),
            confirmed_by_action: ActionId(42),
            confirmed_at: 123,
        });

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(vm.source.source_map.mode, SourceMapModeView::TimeFallback);
    assert_eq!(vm.source.source_map.trust_label, "needs confirm");
    assert_eq!(vm.source.source_map.capture_range_row, ".".repeat(32));
}

#[test]
fn source_map_defaults_to_missing_without_source_graph() {
    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");

    let vm = JamViewModel::build(&session, &ActionQueue::new(), None);

    assert_eq!(vm.source.source_map.mode, SourceMapModeView::Missing);
    assert_eq!(
        vm.source.source_map.capture_hint,
        "cap unavailable | no source graph"
    );
    assert_eq!(vm.source.source_map.capture_range_row, ".".repeat(32));
    assert_eq!(vm.source.source_map.energy_row.chars().count(), 32);
}

#[test]
fn source_map_capture_range_tracks_capture_length_intent() {
    let graph = source_map_test_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.transport.position_beats = 4.0;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(
        vm.source.source_map.capture_range_row,
        "................[=======]......."
    );
}

#[test]
fn source_map_capture_range_starts_at_next_bar_boundary() {
    let graph = source_map_test_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.transport.position_beats = 5.25;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(
        vm.source.source_map.capture_range_row,
        "................[=======]......."
    );
}

#[test]
fn source_map_prefers_bucket_backed_energy_and_peak_rows() {
    let mut graph = source_map_test_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    graph.source_map.buckets = vec![
        source_map_bucket(0.0, 2.0, EnergyClass::Low, SourceMapPeakClass::None),
        source_map_bucket(2.0, 4.0, EnergyClass::High, SourceMapPeakClass::None),
        source_map_bucket(4.0, 6.0, EnergyClass::Peak, SourceMapPeakClass::StrongTransient),
        source_map_bucket(6.0, 8.0, EnergyClass::Medium, SourceMapPeakClass::None),
    ];
    graph.sections[0].energy_class = EnergyClass::Peak;
    graph.sections[1].energy_class = EnergyClass::Low;

    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert_eq!(
        vm.source.source_map.energy_row,
        "▂▂▂▂▂▂▂▂▇▇▇▇▇▇▇▇████████▅▅▅▅▅▅▅▅"
    );
    assert_eq!(
        vm.source.source_map.peak_row,
        "................████████........"
    );
}

#[test]
fn source_map_falls_back_to_sections_when_bucket_evidence_is_missing() {
    let graph = source_map_test_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");

    let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

    assert!(graph.source_map.buckets.is_empty());
    assert_eq!(
        vm.source.source_map.energy_row,
        "▅▅▅▅▅▅▅▅▅▅▅▅▅▅▅▅████████████████"
    );
    assert_eq!(vm.source.source_map.peak_row.chars().next(), Some('█'));
}

fn source_map_test_graph(policy: TimingDegradedPolicy, quality: TimingQuality) -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-map"),
            path: "fixtures/map.wav".into(),
            content_hash: "hash-map".into(),
            duration_seconds: 8.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-05-23T00:00:00Z".into(),
            source_hash: "hash-map".into(),
            analysis_seed: 23,
            run_notes: None,
        },
    );
    graph.sections = vec![
        Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 4.0,
            bar_start: 1,
            bar_end: 2,
            energy_class: EnergyClass::Medium,
            confidence: 0.72,
            tags: Vec::new(),
        },
        Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 4.0,
            end_seconds: 8.0,
            bar_start: 3,
            bar_end: 4,
            energy_class: EnergyClass::Peak,
            confidence: 0.91,
            tags: Vec::new(),
        },
    ];
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.bpm_confidence = 0.9;
    graph.timing.quality = quality;
    graph.timing.degraded_policy = policy;
    graph.timing.primary_hypothesis_id = Some("primary".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "primary".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 120.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.9,
        score: 0.9,
        beat_grid: Vec::new(),
        bar_grid: (0..4)
            .map(|index| BarSpan {
                bar_index: index + 1,
                start_seconds: index as f32 * 2.0,
                end_seconds: (index + 1) as f32 * 2.0,
                downbeat_confidence: 0.9,
                phrase_index: Some(1),
            })
            .collect(),
        phrase_grid: Vec::new(),
        anchors: vec![crate::source_graph::SourceTimingAnchor {
            anchor_id: "kick-1".into(),
            anchor_type: crate::source_graph::SourceTimingAnchorType::Kick,
            time_seconds: 0.0,
            bar_index: Some(1),
            beat_index: Some(1),
            confidence: 0.9,
            strength: 0.9,
            tags: vec!["kick".into()],
        }],
        drift: Vec::new(),
        groove: Vec::new(),
        quality,
        warnings: Vec::new(),
        provenance: vec!["fixture".into()],
    });
    graph
}

fn source_map_bucket(
    start_seconds: f32,
    end_seconds: f32,
    energy_class: EnergyClass,
    peak_class: SourceMapPeakClass,
) -> SourceMapBucket {
    SourceMapBucket {
        start_seconds,
        end_seconds,
        energy_class,
        peak_class,
        confidence: 0.92,
        provenance_refs: vec!["fixture:source-map-bucket".into()],
    }
}
