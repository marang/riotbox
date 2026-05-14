#[derive(Clone, Copy, Debug, PartialEq)]
struct Tr909KickPressureProof {
    applied: bool,
    anchor_count: usize,
    pressure_gain: f32,
    pre_low_band_rms: f32,
    post_low_band_rms: f32,
    low_band_rms_delta: f32,
    low_band_rms_ratio: f32,
    post_peak_abs: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestTr909KickPressureProof {
    applied: bool,
    anchor_count: usize,
    pressure_gain: f32,
    pre_low_band_rms: f32,
    post_low_band_rms: f32,
    low_band_rms_delta: f32,
    low_band_rms_ratio: f32,
    post_peak_abs: f32,
    reason: &'static str,
}

const TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO: f32 = 1.06;
const TR909_KICK_PRESSURE_MAX_PEAK_ABS: f32 = 0.95;

#[allow(dead_code)]
fn render_tr909_source_support(grid: &Grid, profile: SourceAwareTr909Profile) -> Vec<f32> {
    render_tr909_source_support_with_pressure(grid, profile).0
}

fn render_tr909_source_support_with_pressure(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> (Vec<f32>, Tr909KickPressureProof) {
    let mut samples = render_tr909_source_support_legacy(grid, profile);
    let proof = apply_tr909_kick_pressure(&mut samples, grid, profile);
    (samples, proof)
}

fn manifest_tr909_kick_pressure_proof(
    proof: Tr909KickPressureProof,
) -> ManifestTr909KickPressureProof {
    ManifestTr909KickPressureProof {
        applied: proof.applied,
        anchor_count: proof.anchor_count,
        pressure_gain: proof.pressure_gain,
        pre_low_band_rms: proof.pre_low_band_rms,
        post_low_band_rms: proof.post_low_band_rms,
        low_band_rms_delta: proof.low_band_rms_delta,
        low_band_rms_ratio: proof.low_band_rms_ratio,
        post_peak_abs: proof.post_peak_abs,
        reason: proof.reason,
    }
}

fn apply_tr909_kick_pressure(
    samples: &mut [f32],
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> Tr909KickPressureProof {
    let pre = tr909_kick_pressure_metrics(samples, grid);
    let policy = tr909_kick_pressure_policy(profile);
    let mut anchor_count = 0;

    for beat in 0..grid.total_beats {
        let beat_in_bar = beat % grid.beats_per_bar;
        let accent = match beat_in_bar {
            0 => 1.0,
            2 => 0.58,
            _ if profile.support_profile == Tr909SourceSupportProfile::DropDrive => 0.22,
            _ => 0.0,
        };
        if accent <= 0.0 {
            continue;
        }

        let bar = beat / grid.beats_per_bar;
        let bar_pressure = match bar % 4 {
            0 => 1.00,
            1 => 0.86,
            2 => 1.12,
            _ => 0.94,
        };
        render_tr909_kick_pressure_anchor(
            samples,
            grid,
            beat as f32,
            policy,
            accent * bar_pressure,
        );
        anchor_count += 1;
    }

    for sample in &mut *samples {
        *sample = sample.clamp(-TR909_KICK_PRESSURE_MAX_PEAK_ABS, TR909_KICK_PRESSURE_MAX_PEAK_ABS);
    }

    let post = tr909_kick_pressure_metrics(samples, grid);
    let low_band_rms_delta = post.low_band_rms - pre.low_band_rms;
    let low_band_rms_ratio = post.low_band_rms / pre.low_band_rms.max(f32::EPSILON);
    let applied = anchor_count > 0
        && low_band_rms_ratio >= TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO
        && post.peak_abs <= TR909_KICK_PRESSURE_MAX_PEAK_ABS;

    Tr909KickPressureProof {
        applied,
        anchor_count,
        pressure_gain: policy.gain,
        pre_low_band_rms: pre.low_band_rms,
        post_low_band_rms: post.low_band_rms,
        low_band_rms_delta,
        low_band_rms_ratio,
        post_peak_abs: post.peak_abs,
        reason: if applied {
            policy.reason
        } else {
            "tr909_kick_pressure_too_weak"
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct Tr909KickPressurePolicy {
    gain: f32,
    body_hz: f32,
    tail_seconds: f32,
    click_gain: f32,
    reason: &'static str,
}

fn tr909_kick_pressure_policy(profile: SourceAwareTr909Profile) -> Tr909KickPressurePolicy {
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive => Tr909KickPressurePolicy {
            gain: 0.018,
            body_hz: 54.0,
            tail_seconds: 0.180,
            click_gain: 0.006,
            reason: "tr909_low_drive_pressure",
        },
        Tr909SourceSupportProfile::BreakLift => Tr909KickPressurePolicy {
            gain: 0.012,
            body_hz: 58.0,
            tail_seconds: 0.150,
            click_gain: 0.010,
            reason: "tr909_break_lift_pressure",
        },
        Tr909SourceSupportProfile::SteadyPulse => Tr909KickPressurePolicy {
            gain: 0.014,
            body_hz: 52.0,
            tail_seconds: 0.165,
            click_gain: 0.004,
            reason: "tr909_steady_pulse_pressure",
        },
    }
}

fn render_tr909_kick_pressure_anchor(
    samples: &mut [f32],
    grid: &Grid,
    beat_position: f32,
    policy: Tr909KickPressurePolicy,
    accent: f32,
) {
    let start_frame = frames_for_beat_position(grid.bpm, beat_position);
    let frame_count = (policy.tail_seconds * SAMPLE_RATE as f32).round() as usize;
    for offset in 0..frame_count {
        let frame = start_frame.saturating_add(offset);
        if frame >= grid.total_frames {
            break;
        }

        let position = offset as f32 / frame_count.max(1) as f32;
        let pitch_drop = 1.0 - position.min(1.0) * 0.28;
        let phase = offset as f32 * policy.body_hz * pitch_drop / SAMPLE_RATE as f32;
        let body_envelope = (1.0 - position).clamp(0.0, 1.0).powf(2.15);
        let body = (std::f32::consts::TAU * phase).sin() * body_envelope * policy.gain * accent;
        let click = if offset < 72 {
            let click_position = offset as f32 / 72.0;
            (1.0 - click_position) * policy.click_gain * accent
        } else {
            0.0
        };
        let output_index = frame.saturating_mul(usize::from(CHANNEL_COUNT));
        samples[output_index] += body + click;
        samples[output_index + 1] += body * 0.98 + click;
    }
}

#[derive(Clone, Copy, Debug)]
struct Tr909KickPressureMetrics {
    low_band_rms: f32,
    peak_abs: f32,
}

fn tr909_kick_pressure_metrics(samples: &[f32], grid: &Grid) -> Tr909KickPressureMetrics {
    let low_band = signal_metrics_with_grid(
        &one_pole_lowpass(samples, 120.0),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let signal = signal_metrics_with_grid(
        samples,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    Tr909KickPressureMetrics {
        low_band_rms: low_band.rms,
        peak_abs: signal.peak_abs,
    }
}
