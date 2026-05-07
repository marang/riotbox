#[derive(Clone, Copy, Debug, PartialEq)]
struct SourceAwareTr909Profile {
    signal_rms: f32,
    low_band_rms: f32,
    onset_count: usize,
    event_density_per_bar: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    support_profile: Tr909SourceSupportProfile,
    support_context: Tr909SourceSupportContext,
    pattern_adoption: Tr909PatternAdoption,
    phrase_variation: Tr909PhraseVariation,
    drum_bus_level: f32,
    slam_intensity: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestTr909SourceProfile {
    signal_rms: f32,
    low_band_rms: f32,
    onset_count: usize,
    event_density_per_bar: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    support_profile: &'static str,
    support_context: &'static str,
    pattern_adoption: &'static str,
    phrase_variation: &'static str,
    drum_bus_level: f32,
    slam_intensity: f32,
    reason: &'static str,
}

fn derive_source_aware_tr909_profile(samples: &[f32], grid: &Grid) -> SourceAwareTr909Profile {
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
            drum_bus_level: 0.84,
            slam_intensity: 0.22,
            reason: "source_low_drive",
        }
    } else if signal.event_density_per_bar >= 3.0 || spectral.high_band_energy_ratio >= 0.34 {
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

fn manifest_tr909_source_profile(profile: SourceAwareTr909Profile) -> ManifestTr909SourceProfile {
    ManifestTr909SourceProfile {
        signal_rms: profile.signal_rms,
        low_band_rms: profile.low_band_rms,
        onset_count: profile.onset_count,
        event_density_per_bar: profile.event_density_per_bar,
        low_band_energy_ratio: profile.low_band_energy_ratio,
        mid_band_energy_ratio: profile.mid_band_energy_ratio,
        high_band_energy_ratio: profile.high_band_energy_ratio,
        support_profile: profile.support_profile.label(),
        support_context: profile.support_context.label(),
        pattern_adoption: profile.pattern_adoption.label(),
        phrase_variation: profile.phrase_variation.label(),
        drum_bus_level: profile.drum_bus_level,
        slam_intensity: profile.slam_intensity,
        reason: profile.reason,
    }
}
