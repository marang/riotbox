#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceAccentDynamicsProof {
    applied: bool,
    trigger_count: u32,
    distinct_velocity_count: usize,
    min_velocity: f32,
    max_velocity: f32,
    velocity_span: f32,
    min_required_velocity_span: f32,
    source_energy_span: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestW30SourceAccentDynamicsProof {
    pattern_origin: &'static str,
    applied: bool,
    trigger_count: u32,
    distinct_velocity_count: usize,
    min_velocity: f32,
    max_velocity: f32,
    velocity_span: f32,
    min_required_velocity_span: f32,
    source_energy_span: f32,
    reason: &'static str,
}

const W30_SOURCE_ACCENT_MIN_VELOCITY_SPAN: f32 = 0.12;
const W30_SOURCE_ACCENT_MIN_DISTINCT_VELOCITIES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceAccentFeatures {
    velocity: f32,
    source_energy_score: f32,
}

fn manifest_w30_source_accent_dynamics_proof(
    proof: W30SourceAccentDynamicsProof,
) -> ManifestW30SourceAccentDynamicsProof {
    ManifestW30SourceAccentDynamicsProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        applied: proof.applied,
        trigger_count: proof.trigger_count,
        distinct_velocity_count: proof.distinct_velocity_count,
        min_velocity: proof.min_velocity,
        max_velocity: proof.max_velocity,
        velocity_span: proof.velocity_span,
        min_required_velocity_span: proof.min_required_velocity_span,
        source_energy_span: proof.source_energy_span,
        reason: proof.reason,
    }
}

fn w30_source_accent_dynamics_proof(
    events: &[W30SourceTriggerEvent],
) -> W30SourceAccentDynamicsProof {
    if events.is_empty() {
        return W30SourceAccentDynamicsProof {
            applied: false,
            trigger_count: 0,
            distinct_velocity_count: 0,
            min_velocity: 0.0,
            max_velocity: 0.0,
            velocity_span: 0.0,
            min_required_velocity_span: W30_SOURCE_ACCENT_MIN_VELOCITY_SPAN,
            source_energy_span: 0.0,
            reason: "source_accent_dynamics_no_triggers",
        };
    }

    let min_velocity = events
        .iter()
        .map(|event| event.velocity)
        .fold(f32::INFINITY, f32::min);
    let max_velocity = events
        .iter()
        .map(|event| event.velocity)
        .fold(f32::NEG_INFINITY, f32::max);
    let velocity_span = max_velocity - min_velocity;
    let distinct_velocity_count = events
        .iter()
        .map(|event| (event.velocity * 100.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let min_source_energy = events
        .iter()
        .map(|event| event.source_energy_score)
        .fold(f32::INFINITY, f32::min);
    let max_source_energy = events
        .iter()
        .map(|event| event.source_energy_score)
        .fold(f32::NEG_INFINITY, f32::max);
    let source_energy_span = max_source_energy - min_source_energy;
    let applied = distinct_velocity_count >= W30_SOURCE_ACCENT_MIN_DISTINCT_VELOCITIES
        && velocity_span >= W30_SOURCE_ACCENT_MIN_VELOCITY_SPAN;

    W30SourceAccentDynamicsProof {
        applied,
        trigger_count: events.len() as u32,
        distinct_velocity_count,
        min_velocity,
        max_velocity,
        velocity_span,
        min_required_velocity_span: W30_SOURCE_ACCENT_MIN_VELOCITY_SPAN,
        source_energy_span,
        reason: if applied {
            "source_energy_accented_chop_dynamics"
        } else {
            "source_accent_dynamics_too_flat"
        },
    }
}

fn w30_source_accent_features(
    source_window_preview: &W30PreviewSampleWindow,
    source_offset_samples: usize,
) -> W30SourceAccentFeatures {
    let sample_count = source_window_preview
        .sample_count
        .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return W30SourceAccentFeatures {
            velocity: 0.72,
            source_energy_score: 0.0,
        };
    }

    let samples = &source_window_preview.samples[..sample_count];
    let slice_len = (sample_count / W30_SOURCE_SLICE_CHOICE_CANDIDATE_COUNT).max(1);
    let local_rms = circular_slice_rms(samples, source_offset_samples, slice_len);
    let global_rms = rms(samples).max(f32::EPSILON);
    let local_delta = positive_abs_delta_circular(samples, source_offset_samples, slice_len);
    let global_delta = positive_abs_delta(samples).max(f32::EPSILON);
    let energy = ((local_rms / global_rms) * 0.72 + (local_delta / global_delta) * 0.28)
        .clamp(0.0, 1.4);
    let source_position = source_offset_samples as f32 / sample_count.max(1) as f32;
    let source_position_accent =
        (source_position * std::f32::consts::TAU).sin().mul_add(0.5, 0.5) * 0.12;
    W30SourceAccentFeatures {
        velocity: (0.56 + energy * 0.27 + source_position_accent).clamp(0.58, 1.0),
        source_energy_score: energy,
    }
}

fn circular_slice_rms(samples: &[f32], offset: usize, len: usize) -> f32 {
    if samples.is_empty() || len == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for index in 0..len {
        let sample = samples[(offset + index) % samples.len()];
        sum += sample * sample;
    }
    (sum / len as f32).sqrt()
}

fn positive_abs_delta_circular(samples: &[f32], offset: usize, len: usize) -> f32 {
    if samples.len() < 2 || len < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for index in 1..len {
        let previous = samples[(offset + index - 1) % samples.len()];
        let current = samples[(offset + index) % samples.len()];
        total += (current.abs() - previous.abs()).max(0.0);
    }
    total / (len - 1) as f32
}
