//! Existing source-profile policy for the bounded TR-909 offline QA renderer.

use super::config::{CHANNEL_COUNT, SAMPLE_RATE};
use super::grid::Grid;
use super::signal_filter::one_pole_lowpass;
use super::spectral_energy_metrics::spectral_energy_metrics;
use riotbox_audio::runtime::signal_metrics_with_grid;
use riotbox_audio::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909SourceSupportContext,
    Tr909SourceSupportProfile,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SourceAwareTr909Profile {
    pub(super) signal_rms: f32,
    pub(super) low_band_rms: f32,
    pub(super) onset_count: usize,
    pub(super) event_density_per_bar: f32,
    pub(super) low_band_energy_ratio: f32,
    pub(super) mid_band_energy_ratio: f32,
    pub(super) high_band_energy_ratio: f32,
    pub(super) support_profile: Tr909SourceSupportProfile,
    pub(super) support_context: Tr909SourceSupportContext,
    pub(super) pattern_adoption: Tr909PatternAdoption,
    pub(super) phrase_variation: Tr909PhraseVariation,
    pub(super) drum_bus_level: f32,
    pub(super) slam_intensity: f32,
    pub(super) reason: &'static str,
}

pub(super) fn derive_source_aware_tr909_profile(
    samples: &[f32],
    grid: &Grid,
) -> SourceAwareTr909Profile {
    let signal = signal_metrics_with_grid(
        samples,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let low_band = signal_metrics_with_grid(
        &one_pole_lowpass(samples, 165.0),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let spectral = spectral_energy_metrics(samples);

    if spectral.low_band_energy_ratio >= 0.52 || low_band.rms >= signal.rms * 0.60 {
        SourceAwareTr909Profile {
            signal_rms: signal.rms,
            low_band_rms: low_band.rms,
            onset_count: signal.onset_count,
            event_density_per_bar: signal.event_density_per_bar,
            low_band_energy_ratio: spectral.low_band_energy_ratio,
            mid_band_energy_ratio: spectral.mid_band_energy_ratio,
            high_band_energy_ratio: spectral.high_band_energy_ratio,
            support_profile: Tr909SourceSupportProfile::DropDrive,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::MainlineDrive,
            phrase_variation: Tr909PhraseVariation::PhraseDrive,
            drum_bus_level: 0.92,
            slam_intensity: 0.22,
            reason: "source_low_drive",
        }
    } else if signal.event_density_per_bar >= 3.0 && signal.crest_factor >= 3.0 {
        SourceAwareTr909Profile {
            signal_rms: signal.rms,
            low_band_rms: low_band.rms,
            onset_count: signal.onset_count,
            event_density_per_bar: signal.event_density_per_bar,
            low_band_energy_ratio: spectral.low_band_energy_ratio,
            mid_band_energy_ratio: spectral.mid_band_energy_ratio,
            high_band_energy_ratio: spectral.high_band_energy_ratio,
            support_profile: Tr909SourceSupportProfile::BreakLift,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::TakeoverGrid,
            phrase_variation: Tr909PhraseVariation::PhraseLift,
            drum_bus_level: 0.78,
            slam_intensity: 0.34,
            reason: "source_break_lift",
        }
    } else {
        SourceAwareTr909Profile {
            signal_rms: signal.rms,
            low_band_rms: low_band.rms,
            onset_count: signal.onset_count,
            event_density_per_bar: signal.event_density_per_bar,
            low_band_energy_ratio: spectral.low_band_energy_ratio,
            mid_band_energy_ratio: spectral.mid_band_energy_ratio,
            high_band_energy_ratio: spectral.high_band_energy_ratio,
            support_profile: Tr909SourceSupportProfile::SteadyPulse,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::SupportPulse,
            phrase_variation: Tr909PhraseVariation::PhraseAnchor,
            drum_bus_level: 0.70,
            slam_intensity: 0.16,
            reason: "source_steady_pulse",
        }
    }
}
