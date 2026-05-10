const TR909_MAX_GROOVE_OFFSET_MS: f32 = 30.0;
const TR909_MIN_GROOVE_OFFSET_MS: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct Tr909GrooveTimingPolicy {
    applied: bool,
    reason: &'static str,
    offset_ms: f32,
    source_residual_count: usize,
    source_max_abs_offset_ms: f32,
    source_subdivision: Option<&'static str>,
}

fn tr909_groove_timing_policy(
    grid_bpm: GridBpmDecision,
    groove: &ManifestSourceTimingGrooveEvidence,
) -> Tr909GrooveTimingPolicy {
    let base = Tr909GrooveTimingPolicy {
        applied: false,
        reason: "not_source_timing_grid",
        offset_ms: 0.0,
        source_residual_count: groove.primary_groove_residual_count,
        source_max_abs_offset_ms: groove.primary_max_abs_offset_ms,
        source_subdivision: None,
    };

    if grid_bpm.source != GridBpmSource::SourceTiming {
        return base;
    }

    if grid_bpm.reason != GridBpmDecisionReason::SourceTimingReady {
        return Tr909GrooveTimingPolicy {
            reason: "source_timing_not_locked",
            ..base
        };
    }

    let Some(residual) = strongest_groove_residual(groove) else {
        return Tr909GrooveTimingPolicy {
            reason: "no_groove_residuals",
            ..base
        };
    };

    if !residual.offset_ms.is_finite() {
        return Tr909GrooveTimingPolicy {
            reason: "invalid_groove_offset",
            source_subdivision: Some(residual.subdivision),
            ..base
        };
    }

    let offset_ms = residual
        .offset_ms
        .clamp(-TR909_MAX_GROOVE_OFFSET_MS, TR909_MAX_GROOVE_OFFSET_MS);
    if offset_ms.abs() < TR909_MIN_GROOVE_OFFSET_MS {
        return Tr909GrooveTimingPolicy {
            reason: "groove_offset_too_small",
            source_subdivision: Some(residual.subdivision),
            ..base
        };
    }

    Tr909GrooveTimingPolicy {
        applied: true,
        reason: "source_timing_groove_residual",
        offset_ms,
        source_subdivision: Some(residual.subdivision),
        ..base
    }
}

fn strongest_groove_residual(
    groove: &ManifestSourceTimingGrooveEvidence,
) -> Option<&ManifestSourceTimingGrooveResidual> {
    groove
        .primary_groove_preview
        .iter()
        .filter(|residual| residual.confidence.is_finite())
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
}

fn apply_tr909_groove_timing(samples: &[f32], policy: Tr909GrooveTimingPolicy) -> Vec<f32> {
    if !policy.applied || policy.offset_ms.abs() < f32::EPSILON {
        return samples.to_vec();
    }

    let frame_offset =
        (policy.offset_ms * SAMPLE_RATE as f32 / 1000.0).round() as isize;
    shift_interleaved_by_frames(samples, frame_offset)
}

fn shift_interleaved_by_frames(samples: &[f32], frame_offset: isize) -> Vec<f32> {
    if frame_offset == 0 || samples.is_empty() {
        return samples.to_vec();
    }

    let channels = usize::from(CHANNEL_COUNT);
    let sample_offset = frame_offset.unsigned_abs().saturating_mul(channels);
    if sample_offset >= samples.len() {
        return vec![0.0; samples.len()];
    }

    let mut shifted = vec![0.0; samples.len()];
    if frame_offset > 0 {
        shifted[sample_offset..].copy_from_slice(&samples[..samples.len() - sample_offset]);
    } else {
        shifted[..samples.len() - sample_offset].copy_from_slice(&samples[sample_offset..]);
    }
    shifted
}
