use crate::runtime::render_tr909_w30_preview::{
    render_tr909_buffer, render_w30_preview_buffer, render_w30_resample_tap_buffer,
};
use crate::runtime::shared_w30_resample_callback::{
    Tr909CallbackState, W30PreviewCallbackState, W30ResampleTapCallbackState,
};
use crate::runtime::tests::fixture_models::{
    AudioFixtureCase, W30AudioFixtureCase, W30ResampleAudioFixtureCase,
};
use crate::runtime::tests::synthetic_sources::fill_positive_preview_ramp;
use crate::runtime::{render_tr909_offline, render_w30_preview_offline, signal_metrics};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};
use crate::w30::{
    W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
    W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
};

#[test]
fn offline_tr909_render_produces_reviewable_metrics_for_fill() {
    let buffer = render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            drum_bus_level: 0.82,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Tr909RenderState::default()
        },
        44_100,
        2,
        44_100,
    );

    let metrics = signal_metrics(&buffer);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.peak_abs > 0.001);
    assert!(metrics.rms > 0.001);
}

#[test]
fn fixture_backed_tr909_audio_regressions_hold() {
    let fixtures: Vec<AudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/tr909_audio_regression.json"
    ))
    .expect("parse TR-909 audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = Tr909CallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_tr909_buffer(
            &mut buffer,
            44_100,
            2,
            &fixture.render_state.to_realtime(),
            &mut callback_state,
        );

        let active_samples = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak_abs = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let sum = buffer.iter().sum::<f32>();

        assert!(
            active_samples >= fixture.expected.min_active_samples,
            "{} active sample count too low: got {active_samples}",
            fixture.name
        );
        assert!(
            active_samples <= fixture.expected.max_active_samples,
            "{} active sample count too high: got {active_samples}",
            fixture.name
        );
        assert!(
            peak_abs >= fixture.expected.min_peak_abs,
            "{} peak too low: got {peak_abs}",
            fixture.name
        );
        assert!(
            peak_abs <= fixture.expected.max_peak_abs,
            "{} peak too high: got {peak_abs}",
            fixture.name
        );
        if let Some(min_sum) = fixture.expected.min_sum {
            assert!(sum >= min_sum, "{} sum too low: got {sum}", fixture.name);
        }
        if let Some(max_sum) = fixture.expected.max_sum {
            assert!(sum <= max_sum, "{} sum too high: got {sum}", fixture.name);
        }
    }
}

#[test]
fn fixture_backed_w30_preview_audio_regressions_hold() {
    let fixtures: Vec<W30AudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/w30_preview_audio_regression.json"
    ))
    .expect("parse W-30 preview audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = W30PreviewCallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_w30_preview_buffer(
            &mut buffer,
            44_100,
            2,
            &fixture.render_state.to_realtime(),
            &mut callback_state,
        );

        let active_samples = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak_abs = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let sum = buffer.iter().sum::<f32>();
        let rms =
            (buffer.iter().map(|sample| sample * sample).sum::<f32>() / buffer.len() as f32).sqrt();

        assert!(
            active_samples >= fixture.expected.min_active_samples,
            "{} active sample count too low: got {active_samples}",
            fixture.name
        );
        assert!(
            active_samples <= fixture.expected.max_active_samples,
            "{} active sample count too high: got {active_samples}",
            fixture.name
        );
        assert!(
            peak_abs >= fixture.expected.min_peak_abs,
            "{} peak too low: got {peak_abs}",
            fixture.name
        );
        assert!(
            peak_abs <= fixture.expected.max_peak_abs,
            "{} peak too high: got {peak_abs}",
            fixture.name
        );
        if let Some(min_sum) = fixture.expected.min_sum {
            assert!(sum >= min_sum, "{} sum too low: got {sum}", fixture.name);
        }
        if let Some(max_sum) = fixture.expected.max_sum {
            assert!(sum <= max_sum, "{} sum too high: got {sum}", fixture.name);
        }
        if let Some(min_rms) = fixture.expected.min_rms {
            assert!(rms >= min_rms, "{} RMS too low: got {rms}", fixture.name);
        }
        if let Some(max_rms) = fixture.expected.max_rms {
            assert!(rms <= max_rms, "{} RMS too high: got {rms}", fixture.name);
        }
    }
}

#[test]
fn offline_w30_preview_render_produces_reviewable_metrics() {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut samples);

    let buffer = render_w30_preview_offline(
        &W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-01".into()),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: Some(W30PreviewSampleWindow {
                source_start_frame: 0,
                source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
                sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
                samples,
            }),
            pad_playback: None,
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        44_100,
        2,
        256,
    );

    let metrics = signal_metrics(&buffer);

    assert_eq!(buffer.len(), 512);
    assert!(
        metrics.active_samples >= 300,
        "active sample count too low: got {}",
        metrics.active_samples
    );
    assert!(
        (0.07..=0.12).contains(&metrics.rms),
        "unexpected RMS: got {}",
        metrics.rms
    );
    assert!(
        (38.0..=58.0).contains(&metrics.sum),
        "unexpected sum: got {}",
        metrics.sum
    );
    assert!(
        (0.08..=0.12).contains(&metrics.peak_abs),
        "unexpected peak: got {}",
        metrics.peak_abs
    );
}

#[test]
fn fixture_backed_w30_resample_audio_regressions_hold() {
    let fixtures: Vec<W30ResampleAudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/w30_resample_audio_regression.json"
    ))
    .expect("parse W-30 resample audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = W30ResampleTapCallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_w30_resample_tap_buffer(
            &mut buffer,
            44_100,
            2,
            &fixture.render_state.to_realtime(),
            &mut callback_state,
        );

        let active_samples = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak_abs = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

        assert!(
            active_samples >= fixture.expected.min_active_samples,
            "{} active sample count too low: got {active_samples}",
            fixture.name
        );
        assert!(
            active_samples <= fixture.expected.max_active_samples,
            "{} active sample count too high: got {active_samples}",
            fixture.name
        );
        assert!(
            peak_abs >= fixture.expected.min_peak_abs,
            "{} peak too low: got {peak_abs}",
            fixture.name
        );
        assert!(
            peak_abs <= fixture.expected.max_peak_abs,
            "{} peak too high: got {peak_abs}",
            fixture.name
        );
    }
}
