use std::path::Path;

use riotbox_audio::{runtime::apply_master_bus_soft_limiter, source_audio::SourceAudioCache};
use serde::Serialize;

use super::{MixPolicy, one_pole_lowpass};

pub(super) const PRODUCT_STEM_RECONSTRUCTION_SCHEMA: &str =
    "riotbox.product_stem_reconstruction.v1";
pub(super) const PRODUCT_STEM_RECONSTRUCTION_RULE: &str = "pcm_sum_v1";
pub(super) const PRODUCT_STEM_PCM_MAX_ABS_ERROR: f32 = 3.0 / 32_768.0;
pub(super) const PRODUCT_STEM_PCM_MAX_RMS_ERROR: f32 = 1.5 / 32_768.0;

#[derive(Clone, Debug)]
pub(super) struct ProductStemContributionRender {
    pub(super) drums: Vec<f32>,
    pub(super) music: Vec<f32>,
    pub(super) bass: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct ProductStemReconstructionReport {
    pub(super) schema: &'static str,
    pub(super) rule: &'static str,
    pub(super) passed: bool,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) frame_count: u64,
    pub(super) max_abs_error: f32,
    pub(super) rms_error: f32,
    pub(super) max_allowed_abs_error: f32,
    pub(super) max_allowed_rms_error: f32,
}

pub(super) fn render_product_stem_contributions(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    full_mix: &[f32],
    policy: MixPolicy,
) -> Result<ProductStemContributionRender, String> {
    if tr909.len() != mc202.len() || tr909.len() != w30.len() || tr909.len() != full_mix.len() {
        return Err("product stem inputs must have identical sample counts".into());
    }

    let tr909_low = one_pole_lowpass(tr909, 165.0);
    let mc202_low = one_pole_lowpass(mc202, 165.0);
    let mut drums = Vec::with_capacity(full_mix.len());
    let mut music = Vec::with_capacity(full_mix.len());
    let mut bass = Vec::with_capacity(full_mix.len());

    for index in 0..full_mix.len() {
        let drum_component =
            tr909[index] * policy.tr909_gain + tr909_low[index] * policy.tr909_low_gain;
        let bass_component =
            mc202[index] * policy.mc202_gain + mc202_low[index] * policy.mc202_low_gain;
        let music_component = w30[index] * policy.w30_gain;
        let full_sample = full_mix[index];

        let drum_contribution =
            shapley_contribution(drum_component, bass_component, music_component, policy);
        let bass_contribution =
            shapley_contribution(bass_component, drum_component, music_component, policy);
        let music_contribution = full_sample - drum_contribution - bass_contribution;

        if !drum_contribution.is_finite()
            || !bass_contribution.is_finite()
            || !music_contribution.is_finite()
        {
            return Err(format!(
                "product stem contribution became non-finite at sample {index}"
            ));
        }
        let contribution_peak = drum_contribution
            .abs()
            .max(bass_contribution.abs())
            .max(music_contribution.abs());
        if contribution_peak > 1.0 {
            return Err(format!(
                "product stem contribution exceeded PCM range at sample {index}: {contribution_peak}"
            ));
        }

        drums.push(drum_contribution);
        bass.push(bass_contribution);
        music.push(music_contribution);
    }

    Ok(ProductStemContributionRender { drums, music, bass })
}

fn shapley_contribution(owned: f32, other_a: f32, other_b: f32, policy: MixPolicy) -> f32 {
    let owned_alone = product_bus_sample(owned, policy);
    let with_other_a = product_bus_sample(owned + other_a, policy);
    let with_other_b = product_bus_sample(owned + other_b, policy);
    let other_a_alone = product_bus_sample(other_a, policy);
    let other_b_alone = product_bus_sample(other_b, policy);
    let all = product_bus_sample(owned + other_a + other_b, policy);
    let others = product_bus_sample(other_a + other_b, policy);

    owned_alone / 3.0
        + ((with_other_a - other_a_alone) + (with_other_b - other_b_alone)) / 6.0
        + (all - others) / 3.0
}

fn product_bus_sample(component_sum: f32, policy: MixPolicy) -> f32 {
    let mut value = (component_sum * policy.drive).tanh() * policy.output_gain;
    apply_master_bus_soft_limiter(std::slice::from_mut(&mut value));
    value
}

pub(super) fn validate_written_product_stem_reconstruction(
    drums_path: &Path,
    music_path: &Path,
    bass_path: &Path,
    full_mix_path: &Path,
) -> Result<ProductStemReconstructionReport, String> {
    let drums = SourceAudioCache::load_pcm_wav(drums_path).map_err(|error| error.to_string())?;
    let music = SourceAudioCache::load_pcm_wav(music_path).map_err(|error| error.to_string())?;
    let bass = SourceAudioCache::load_pcm_wav(bass_path).map_err(|error| error.to_string())?;
    let full_mix =
        SourceAudioCache::load_pcm_wav(full_mix_path).map_err(|error| error.to_string())?;

    let expected_format = (
        full_mix.sample_rate,
        full_mix.channel_count,
        full_mix.frame_count(),
    );
    for (role, stem) in [
        ("stem_drums", &drums),
        ("stem_music", &music),
        ("stem_bass", &bass),
    ] {
        let actual_format = (stem.sample_rate, stem.channel_count, stem.frame_count());
        if actual_format != expected_format {
            return Err(format!(
                "{role} format/grid mismatch: expected {expected_format:?}, got {actual_format:?}"
            ));
        }
    }

    let mut max_abs_error = 0.0_f32;
    let mut squared_error_sum = 0.0_f64;
    let sample_count = full_mix.interleaved_samples().len();
    for index in 0..sample_count {
        let reconstructed = drums.interleaved_samples()[index]
            + music.interleaved_samples()[index]
            + bass.interleaved_samples()[index];
        let error = reconstructed - full_mix.interleaved_samples()[index];
        max_abs_error = max_abs_error.max(error.abs());
        squared_error_sum += f64::from(error) * f64::from(error);
    }
    let rms_error = if sample_count == 0 {
        0.0
    } else {
        (squared_error_sum / sample_count as f64).sqrt() as f32
    };
    let passed = max_abs_error <= PRODUCT_STEM_PCM_MAX_ABS_ERROR
        && rms_error <= PRODUCT_STEM_PCM_MAX_RMS_ERROR;

    Ok(ProductStemReconstructionReport {
        schema: PRODUCT_STEM_RECONSTRUCTION_SCHEMA,
        rule: PRODUCT_STEM_RECONSTRUCTION_RULE,
        passed,
        sample_rate_hz: full_mix.sample_rate,
        channel_count: full_mix.channel_count,
        frame_count: full_mix.frame_count() as u64,
        max_abs_error,
        rms_error,
        max_allowed_abs_error: PRODUCT_STEM_PCM_MAX_ABS_ERROR,
        max_allowed_rms_error: PRODUCT_STEM_PCM_MAX_RMS_ERROR,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapley_product_stems_reconstruct_the_existing_nonlinear_mix() {
        let policy = MixPolicy {
            tr909_gain: 1.15,
            tr909_low_gain: 0.46,
            mc202_gain: 0.21,
            mc202_low_gain: 0.065,
            w30_gain: 1.50,
            drive: 2.18,
            output_gain: 0.94,
        };
        let tr909 = signal(1_024, 0.37, 0.71);
        let mc202 = signal(1_024, 0.19, 1.13);
        let w30 = signal(1_024, 0.41, 0.29);
        let (full_mix, _) =
            super::super::render_mix_with_master_bus_report(&tr909, &mc202, &w30, policy);

        let stems = render_product_stem_contributions(&tr909, &mc202, &w30, &full_mix, policy)
            .expect("render product stem contributions");

        for (index, full_sample) in full_mix.iter().copied().enumerate() {
            let reconstructed = stems.drums[index] + stems.music[index] + stems.bass[index];
            assert!((reconstructed - full_sample).abs() <= f32::EPSILON * 2.0);
        }
    }

    fn signal(len: usize, gain: f32, phase_scale: f32) -> Vec<f32> {
        (0..len)
            .map(|index| {
                let phase = index as f32 * phase_scale;
                (phase.sin() * 0.72 + (phase * 0.31).cos() * 0.28) * gain
            })
            .collect()
    }
}
