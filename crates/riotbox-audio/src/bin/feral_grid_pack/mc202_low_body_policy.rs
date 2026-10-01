use super::{
    mc202_source_contour::{Mc202SourceContourProfile, mc202_source_low_dominance},
    signal_filter::one_pole_lowpass,
};
use riotbox_audio::mc202::Mc202ContourHint;

fn mc202_low_body_emphasis(source_contour: Mc202SourceContourProfile) -> f32 {
    if source_contour.contour_hint != Mc202ContourHint::Drop {
        return 0.0;
    }

    let low_dominance = mc202_source_low_dominance(source_contour);
    ((low_dominance - 0.22) * 0.72).clamp(0.0, 0.38)
}

pub(super) fn apply_mc202_low_body_emphasis(
    samples: &mut [f32],
    source_contour: Mc202SourceContourProfile,
) -> f32 {
    let emphasis = mc202_low_body_emphasis(source_contour);
    if emphasis <= 0.0 || samples.is_empty() {
        return 0.0;
    }

    let cutoff_hz = 132.0 + source_contour.low_band_energy_ratio.clamp(0.0, 1.0) * 42.0;
    let low_body = one_pole_lowpass(samples, cutoff_hz);
    let dry_gain = 1.0 - emphasis * 0.60;
    let low_gain = 1.0 + emphasis * 1.45;

    for (sample, low_sample) in samples.iter_mut().zip(low_body.iter()) {
        *sample = (*sample * dry_gain + *low_sample * low_gain * emphasis).clamp(-0.98, 0.98);
    }

    emphasis
}
