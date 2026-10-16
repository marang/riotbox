//! Stopped active-pad controls delivered through the exact production callback.
use super::w30_fixture::{marked_preview, prepared_with_preview, process, warm};
use crate::runtime::{
    AudioRuntimeShell, AudioTransportPositionUpdate,
    shared_transport_tr909::AudioRuntimeSharedState,
};

#[test]
fn preserve_stop_fades_and_latches_then_trigger_and_material_updates_remain_live() {
    let (state, shell, mut callback) = prepared_with_preview(marked_preview());
    warm(&shell, &mut callback);
    stop_and_latch(&state, &shell, &mut callback);
    trigger_replace_and_resume(&state, &shell, &mut callback);
}

fn stop_and_latch(
    state: &AudioRuntimeSharedState,
    shell: &AudioRuntimeShell,
    callback: &mut impl FnMut(&mut [f32]),
) {
    let material = state.w30_preview.try_snapshot().unwrap();
    let mut output = [0.0; 50];
    // Resolve the ordinary beat1.5 pad trigger before checking the stop fade.
    process(callback, &mut output);
    assert!(output.iter().any(|sample| *sample > 0.0));
    shell
        .update_transport_state(false, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    assert!(
        shell.timing_snapshot().is_some(),
        "Preserve creates no pending request"
    );
    process(callback, &mut output);
    // Existing stop fade is sample_rate/200: five mono frames at 1000Hz.
    assert!(output[..5].iter().any(|sample| *sample > 0.0));
    assert!(output[5..].iter().all(|sample| *sample == 0.0));
    let stopped = shell.timing_snapshot().unwrap();
    assert!(!stopped.is_transport_running);
    assert!((stopped.position_beats - 1.6).abs() < 1e-11);
    process(callback, &mut output);
    assert!(output.iter().all(|sample| *sample == 0.0));
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
}

fn trigger_replace_and_resume(
    state: &AudioRuntimeSharedState,
    shell: &AudioRuntimeShell,
    callback: &mut impl FnMut(&mut [f32]),
) {
    let mut output = [0.0; 50];

    // The normal public manual trigger can clear a stopped latch without a
    // transport seek or a new material identity.
    let mut changed = marked_preview();
    changed.trigger_revision += 1;
    shell.update_w30_preview_render_state(&changed);
    let retriggered_material = state.w30_preview.try_snapshot().unwrap();
    process(callback, &mut output);
    assert_eq!(output[0], 0.0, "an actual trigger retains its attack reset");
    assert!(output[1..].iter().all(|sample| *sample > 0.0));
    assert_eq!(
        state.w30_preview.try_snapshot().unwrap(),
        retriggered_material
    );
    assert!(!shell.timing_snapshot().unwrap().is_transport_running);

    // Change only generated PCM, retaining trigger and all pad metadata. The
    // callback snapshot cache must publish new material without a fake seek or
    // trigger, and the existing playback cursor/attack must remain continuous.
    for sample in &mut changed.pad_playback.as_mut().unwrap().samples {
        *sample = -*sample;
    }
    shell.update_w30_preview_render_state(&changed);
    let replacement = state.w30_preview.try_snapshot().unwrap();
    assert_eq!(
        replacement.trigger_revision,
        retriggered_material.trigger_revision
    );
    process(callback, &mut output);
    assert!(output.iter().all(|sample| *sample < 0.0));
    assert!((shell.timing_snapshot().unwrap().position_beats - 1.6).abs() < 1e-11);

    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    process(callback, &mut output);
    assert!(
        output.iter().all(|sample| *sample < 0.0),
        "Play must not cold-reset the active pad attack"
    );
    let resumed = shell.timing_snapshot().unwrap();
    assert!(resumed.is_transport_running);
    assert!((resumed.position_beats - 1.7).abs() < 1e-11);
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), replacement);
}
