//! Lineage-ready generated resample PCM in the exact production callback.
use super::w30_fixture::{assert_pcm_eq, prepared_with_resample, process, reposition, warm};
use crate::{
    runtime::AudioTransportPositionUpdate,
    w30::{
        W30_RESAMPLE_SOURCE_WINDOW_LEN, W30PreviewRenderMode, W30ResampleSourceWindow,
        W30ResampleTapAvailability, W30ResampleTapMode, W30ResampleTapRouting,
        W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};

#[test]
fn resample_seek_preserves_cursor_and_envelope_until_its_next_destination_trigger() {
    let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.04 + (index % 71) as f32 * 0.002;
    }
    let tap = W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        availability: W30ResampleTapAvailability::SourceAudioReady,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_capture_id: Some("generated-resample-metadata".into()),
        source_audio: Some(Box::new(W30ResampleSourceWindow {
            source_start_frame: 0,
            source_sample_rate: 1000,
            source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
            sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
            samples,
        })),
        lineage_capture_count: 1,
        generation_depth: 1,
        music_bus_level: 0.6,
        grit_level: 0.0,
        ..W30ResampleTapState::default()
    };
    let (control_state, control_shell, mut control_callback) = prepared_with_resample(tap.clone());
    let (state, shell, mut callback) = prepared_with_resample(tap);
    let material = state.w30_resample_tap.try_snapshot().unwrap();
    assert_eq!(
        state.w30_preview.try_snapshot().unwrap().mode,
        W30PreviewRenderMode::Idle
    );
    warm(&control_shell, &mut control_callback);
    warm(&shell, &mut callback);
    control_shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    shell
        .update_transport_state(true, 120.0, reposition(5.95))
        .unwrap();
    assert!(control_shell.timing_snapshot().is_some());
    assert!(shell.timing_snapshot().is_none());
    let mut control = [0.0; 50];
    let mut candidate = [0.0; 50];
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);

    // Preserve source/grain cursor and decay until the requested phase reaches
    // raw resample step6, which owns its ordinary envelope/grain retrigger.
    assert_pcm_eq(&candidate[..16], &control[..16]);
    assert!(candidate[..16].iter().all(|sample| *sample > 0.0));
    assert!(
        candidate[35..]
            .iter()
            .zip(&control[35..])
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "the actual resample output must follow the destination's next trigger"
    );
    assert!((shell.timing_snapshot().unwrap().position_beats - 6.05).abs() < 1e-11);
    assert!((control_shell.timing_snapshot().unwrap().position_beats - 1.6).abs() < 1e-11);
    assert_eq!(state.w30_resample_tap.try_snapshot().unwrap(), material);
    assert_eq!(
        control_state.w30_resample_tap.try_snapshot().unwrap(),
        material
    );
}
