use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::tests::signal_test_helpers::{test_band_rms, tr909_low_band_rms};
use crate::runtime::w30_tr909_signal_helpers::render_gain;
use crate::runtime::{render_tr909_offline, signal_delta_metrics, signal_metrics};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};

#[test]
fn break_reinforce_slam_carries_physical_transient_and_body() {
    let buffer = render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            drum_bus_level: 0.80,
            slam_intensity: 0.66,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
            ..Tr909RenderState::default()
        },
        48_000,
        2,
        48_000,
    );

    let metrics = signal_metrics(&buffer);

    assert!(
        metrics.peak_abs > 0.22,
        "drum peak stayed too polite: {metrics:?}"
    );
    assert!(
        metrics.rms > 0.012,
        "drum body stayed too weak: {metrics:?}"
    );
    assert!(
        metrics.active_sample_ratio > 0.12,
        "drum envelope collapsed to click-sized support: {metrics:?}"
    );
    assert_eq!(metrics.clip_count, 0, "drum support clipped: {metrics:?}");
}

#[test]
fn fill_slam_intensity_creates_bounded_punch_not_just_pitch_drift() {
    let base = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
        drum_bus_level: 0.82,
        slam_intensity: 0.70,
        is_transport_running: true,
        tempo_bpm: 132.0,
        position_beats: 13.0,
        ..Tr909RenderState::default()
    };
    let slammed = Tr909RenderState {
        slam_enabled: true,
        slam_intensity: 0.85,
        ..base.clone()
    };

    let before = render_tr909_offline(&base, 48_000, 2, 48_000);
    let after = render_tr909_offline(&slammed, 48_000, 2, 48_000);
    let before_metrics = signal_metrics(&before);
    let after_metrics = signal_metrics(&after);
    let delta = signal_delta_metrics(&before, &after);
    let before_low = test_band_rms(&before, 40.0, 120.0, 48_000, 2);
    let after_low = test_band_rms(&after, 40.0, 120.0, 48_000, 2);
    let before_attack = test_band_rms(&before, 2_000.0, 10_000.0, 48_000, 2);
    let after_attack = test_band_rms(&after, 2_000.0, 10_000.0, 48_000, 2);

    assert!(
        after_metrics.rms > before_metrics.rms * 1.25,
        "fill slam did not add enough body: before={before_metrics:?} after={after_metrics:?}"
    );
    assert!(
        delta.rms > 0.015 && delta.peak_abs > 0.08,
        "fill slam remained too subtle: {delta:?}"
    );
    assert!(
        after_low > before_low * 3.0,
        "fill slam did not add absolute kick body: before={before_low:.6} after={after_low:.6}"
    );
    assert!(
        after_low / after_attack > before_low / before_attack * 1.20,
        "fill slam stayed a gain-only change instead of shifting toward kick body: low before/after={before_low:.6}/{after_low:.6} attack before/after={before_attack:.6}/{after_attack:.6}"
    );
    assert_eq!(
        after_metrics.clip_count, 0,
        "fill slam clipped: {after_metrics:?}"
    );
}

#[test]
fn cursor_20_unslammed_fill_keeps_headroom_below_break_reinforcement() {
    let fill = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        takeover_profile: None,
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        source_bar_grid_anchor_position_beats: None,
    };
    let break_reinforce = RealtimeTr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        ..fill
    };

    let fill_gain = render_gain(&fill);
    let break_gain = render_gain(&break_reinforce);

    assert!(
        fill_gain < break_gain,
        "the unslammed fill must not inherit the BreakReinforce pressure floor: fill={fill_gain:.6} break={break_gain:.6}"
    );
    assert!(fill_gain <= 0.46);
}

#[test]
fn explicit_break_slam_adds_low_mid_body_and_attack_without_gain_only_fallback() {
    let base = Tr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        drum_bus_level: 0.82,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 132.0,
        position_beats: 16.0,
        ..Tr909RenderState::default()
    };
    let slammed = Tr909RenderState {
        slam_enabled: true,
        slam_intensity: 0.85,
        ..base.clone()
    };

    let before = render_tr909_offline(&base, 48_000, 2, 48_000);
    let after = render_tr909_offline(&slammed, 48_000, 2, 48_000);
    let before_metrics = signal_metrics(&before);
    let after_metrics = signal_metrics(&after);
    let delta = signal_delta_metrics(&before, &after);
    let before_low_mid = tr909_low_band_rms(&before, 48_000, 2);
    let after_low_mid = tr909_low_band_rms(&after, 48_000, 2);

    assert!(
        after_low_mid > before_low_mid * 1.25,
        "break slam did not add low-mid body: before={before_low_mid:.6} after={after_low_mid:.6}"
    );
    assert!(
        delta.rms > 0.02 && delta.peak_abs > 0.12,
        "break slam remained a subtle control change: {delta:?}"
    );
    assert!(
        after_metrics.active_sample_ratio > before_metrics.active_sample_ratio * 1.10,
        "break slam remained a click instead of sustained punch: before={before_metrics:?} after={after_metrics:?}"
    );
    assert_eq!(
        after_metrics.clip_count, 0,
        "break slam clipped: {after_metrics:?}"
    );
}
