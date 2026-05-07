#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridBpmSource {
    UserOverride,
    SourceTiming,
    StaticDefault,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GridBpmDecision {
    bpm: f32,
    source: GridBpmSource,
    source_primary_bpm: Option<f32>,
    source_delta_bpm: Option<f32>,
}

fn choose_grid_bpm(
    args: &Args,
    timing_readiness: &SourceTimingProbeReadinessReport,
) -> GridBpmDecision {
    let source_primary_bpm = timing_readiness.primary_bpm;
    let source_delta_bpm = source_primary_bpm.map(|bpm| (args.bpm - bpm).abs());

    if args.bpm_overridden {
        return GridBpmDecision {
            bpm: args.bpm,
            source: GridBpmSource::UserOverride,
            source_primary_bpm,
            source_delta_bpm,
        };
    }

    if timing_readiness.readiness == SourceTimingProbeReadinessStatus::Ready
        && let Some(bpm) = source_primary_bpm.filter(|bpm| bpm.is_finite() && *bpm > 0.0)
    {
        return GridBpmDecision {
            bpm,
            source: GridBpmSource::SourceTiming,
            source_primary_bpm,
            source_delta_bpm: Some(0.0),
        };
    }

    GridBpmDecision {
        bpm: args.bpm,
        source: GridBpmSource::StaticDefault,
        source_primary_bpm,
        source_delta_bpm,
    }
}

fn grid_bpm_source_label(source: GridBpmSource) -> &'static str {
    match source {
        GridBpmSource::UserOverride => "user_override",
        GridBpmSource::SourceTiming => "source_timing",
        GridBpmSource::StaticDefault => "static_default",
    }
}

fn source_timing_bpm_agrees(delta_bpm: Option<f32>) -> Option<bool> {
    delta_bpm.map(|delta| delta <= SOURCE_TIMING_BPM_MATCH_TOLERANCE)
}
