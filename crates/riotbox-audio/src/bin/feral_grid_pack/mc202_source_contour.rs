use super::{
    config::{CHANNEL_COUNT, SAMPLE_RATE},
    grid::Grid,
    spectral_energy_metrics::spectral_energy_metrics,
};
use riotbox_audio::{
    mc202::{Mc202ContourHint, Mc202NoteBudget},
    runtime::signal_metrics_with_grid,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Mc202SourceContourProfile {
    pub(super) contour_hint: Mc202ContourHint,
    pub(super) note_budget: Mc202NoteBudget,
    pub(super) touch_boost: f32,
    pub(super) music_bus_boost: f32,
    pub(super) low_band_energy_ratio: f32,
    pub(super) mid_band_energy_ratio: f32,
    pub(super) high_band_energy_ratio: f32,
    pub(super) event_density_per_bar: f32,
    pub(super) reason: &'static str,
}

pub(super) const MC202_REASON_LOW_SECTION_DROP_CONTOUR: &str = "source_low_section_drop_contour";

const MC202_REASON_BUSY_SECTION_LIFT_CONTOUR: &str = "source_busy_section_lift_contour";

pub(super) const MC202_REASON_MID_SECTION_HOLD_CONTOUR: &str = "source_mid_section_hold_contour";

impl Mc202SourceContourProfile {
    pub(super) fn from_source_window(samples: &[f32], grid: &Grid) -> Self {
        let spectral = spectral_energy_metrics(samples);
        let signal = signal_metrics_with_grid(
            samples,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        );

        if spectral.low_band_energy_ratio >= spectral.high_band_energy_ratio
            && spectral.low_band_energy_ratio >= spectral.mid_band_energy_ratio
        {
            Self {
                contour_hint: Mc202ContourHint::Drop,
                note_budget: Mc202NoteBudget::Balanced,
                touch_boost: 0.055,
                music_bus_boost: 0.040,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: MC202_REASON_LOW_SECTION_DROP_CONTOUR,
            }
        } else if signal.event_density_per_bar >= 3.0
            || spectral.high_band_energy_ratio >= spectral.mid_band_energy_ratio
        {
            Self {
                contour_hint: Mc202ContourHint::Lift,
                note_budget: Mc202NoteBudget::Push,
                touch_boost: 0.045,
                music_bus_boost: 0.035,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: MC202_REASON_BUSY_SECTION_LIFT_CONTOUR,
            }
        } else {
            Self {
                contour_hint: Mc202ContourHint::Hold,
                note_budget: Mc202NoteBudget::Sparse,
                touch_boost: 0.035,
                music_bus_boost: 0.025,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: MC202_REASON_MID_SECTION_HOLD_CONTOUR,
            }
        }
    }
}

pub(super) fn mc202_source_low_dominance(source_contour: Mc202SourceContourProfile) -> f32 {
    (source_contour.low_band_energy_ratio
        - source_contour
            .mid_band_energy_ratio
            .max(source_contour.high_band_energy_ratio))
    .max(0.0)
}
