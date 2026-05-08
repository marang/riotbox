use riotbox_core::source_graph::{
    BarSpan, BeatPoint, GrooveResidual, GrooveSubdivision, MeterHint, PhraseSpan, SourceGraph,
    SourceTimingAnchor, SourceTimingAnchorType, TimingHypothesis, TimingHypothesisKind,
    TimingQuality,
};

pub(super) fn attach_locked_timing_grid(graph: &mut SourceGraph, bpm: f32) {
    let beat_seconds = 60.0 / bpm;
    graph.timing.meter_hint = Some(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    graph.timing.beat_grid = (0..16)
        .map(|beat_index| BeatPoint {
            beat_index,
            time_seconds: beat_index as f32 * beat_seconds,
            confidence: 0.92,
        })
        .collect();
    graph.timing.bar_grid = (0..4)
        .map(|bar_index| {
            let start_beat = bar_index * 4;
            BarSpan {
                bar_index,
                start_seconds: start_beat as f32 * beat_seconds,
                end_seconds: (start_beat + 4) as f32 * beat_seconds,
                downbeat_confidence: 0.92,
                phrase_index: Some(0),
            }
        })
        .collect();
    graph.timing.phrase_grid = vec![PhraseSpan {
        phrase_index: 0,
        start_bar: 0,
        end_bar: 4,
        confidence: 0.92,
    }];
    graph.timing.primary_hypothesis_id = Some("feral-grid-locked-primary".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "feral-grid-locked-primary".into(),
        kind: TimingHypothesisKind::Primary,
        bpm,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.92,
        score: 0.92,
        beat_grid: graph.timing.beat_grid.clone(),
        bar_grid: graph.timing.bar_grid.clone(),
        phrase_grid: graph.timing.phrase_grid.clone(),
        anchors: locked_grid_anchors(beat_seconds),
        drift: Vec::new(),
        groove: locked_grid_groove(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec!["user_session_observer_probe.locked_timing_grid".into()],
    }];
}

fn locked_grid_groove() -> Vec<GrooveResidual> {
    vec![
        GrooveResidual {
            subdivision: GrooveSubdivision::Eighth,
            offset_ms: -6.0,
            confidence: 0.78,
        },
        GrooveResidual {
            subdivision: GrooveSubdivision::Sixteenth,
            offset_ms: 3.5,
            confidence: 0.66,
        },
    ]
}

fn locked_grid_anchors(beat_seconds: f32) -> Vec<SourceTimingAnchor> {
    let mut anchors = Vec::new();
    for zero_based_bar_index in 0..4 {
        let bar_label = zero_based_bar_index + 1;
        let bar_start_beat = zero_based_bar_index * 4;
        anchors.push(grid_anchor(
            format!("locked-kick-b{bar_label}"),
            SourceTimingAnchorType::Kick,
            bar_start_beat,
            zero_based_bar_index,
            beat_seconds,
            1.0,
            ["kick_anchor", "downbeat", "grid_aligned"],
        ));
        anchors.push(grid_anchor(
            format!("locked-backbeat-b{bar_label}-2"),
            SourceTimingAnchorType::Backbeat,
            bar_start_beat + 1,
            zero_based_bar_index,
            beat_seconds,
            0.82,
            ["backbeat_anchor", "snare_style", "beat_in_bar_2"],
        ));
        anchors.push(grid_anchor(
            format!("locked-transient-b{bar_label}-3"),
            SourceTimingAnchorType::TransientCluster,
            bar_start_beat + 2,
            zero_based_bar_index,
            beat_seconds,
            0.58,
            ["transient_cluster", "beat_in_bar_3", "grid_aligned"],
        ));
        anchors.push(grid_anchor(
            format!("locked-backbeat-b{bar_label}-4"),
            SourceTimingAnchorType::Backbeat,
            bar_start_beat + 3,
            zero_based_bar_index,
            beat_seconds,
            0.84,
            ["backbeat_anchor", "snare_style", "beat_in_bar_4"],
        ));
    }
    anchors
}

fn grid_anchor<const N: usize>(
    anchor_id: String,
    anchor_type: SourceTimingAnchorType,
    zero_based_beat_index: u32,
    zero_based_bar_index: u32,
    beat_seconds: f32,
    strength: f32,
    tags: [&str; N],
) -> SourceTimingAnchor {
    SourceTimingAnchor {
        anchor_id,
        anchor_type,
        time_seconds: zero_based_beat_index as f32 * beat_seconds,
        bar_index: Some(zero_based_bar_index),
        beat_index: Some(zero_based_beat_index),
        confidence: 0.92,
        strength,
        tags: tags.into_iter().map(String::from).collect(),
    }
}
