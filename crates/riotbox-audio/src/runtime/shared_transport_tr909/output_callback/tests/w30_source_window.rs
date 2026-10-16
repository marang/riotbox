//! Source-window phase and cursor continuity in the exact production callback.
use super::w30_fixture::{
    assert_pcm_eq, generated_preview, prepared_with_preview, process, reposition, warm,
};
use crate::{
    runtime::AudioTransportPositionUpdate,
    w30::{W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewSampleWindow, W30PreviewSourceProfile},
};

#[test]
fn source_window_seek_preserves_playback_until_its_next_destination_step() {
    let mut preview = generated_preview(false);
    preview.pad_playback = None;
    preview.source_profile = Some(W30PreviewSourceProfile::PinnedRecall);
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.04 + (index % 23) as f32 * 0.003;
    }
    preview.source_window_preview = Some(W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples,
    });
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(preview.clone());
    let (state, shell, mut callback) = prepared_with_preview(preview);
    let material = state.w30_preview.try_snapshot().unwrap();
    assert_eq!(material.pad_playback.sample_count, 0);
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

    // No immediate cursor/envelope retrigger: the marker prefix is identical.
    // Only the seeked clock crosses a natural PinnedRecall step at beat6.
    assert_pcm_eq(&candidate[..16], &control[..16]);
    assert!(candidate[..16].iter().any(|sample| *sample > 0.0));
    assert!(
        candidate[35..]
            .iter()
            .zip(&control[35..])
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "destination step must retrigger source-window playback after the continuous prefix"
    );
    assert!((shell.timing_snapshot().unwrap().position_beats - 6.05).abs() < 1e-11);
    assert!((control_shell.timing_snapshot().unwrap().position_beats - 1.6).abs() < 1e-11);
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
}
