use crate::runtime::tests::signal_test_helpers::{
    test_band_rms, test_max_frame_delta, test_signal_rms,
};
use crate::runtime::tests::tr909_fill_fixtures::{
    cursor_20_phrase_drive_fill_render, render_candidate6_composite_control,
    render_isolated_fill_voice_component, test_fill_recipe_trigger,
};
use crate::runtime::tr909_fill_voice;
use crate::runtime::w30_tr909_signal_helpers::{render_subdivision, trigger_envelope};
use crate::runtime::{master_bus_limiter_threshold, render_tr909_offline, signal_metrics};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};

#[test]
fn fill_kick_snare_and_hat_tails_overlap_without_truncating_each_other() {
    let render = cursor_20_phrase_drive_fill_render();
    let mut voices = tr909_fill_voice::Tr909FillVoiceState::default();
    voices.start();
    let sample_rate = 48_000;
    let frames_per_step =
        (sample_rate as f32 * 60.0 / render.tempo_bpm / render_subdivision(&render) as f32).round()
            as usize;

    voices.trigger(
        test_fill_recipe_trigger(&render, 18),
        trigger_envelope(&render),
        18,
    );
    for _ in 0..frames_per_step {
        let _ = voices.render_sample(&render, sample_rate);
    }
    voices.trigger(
        test_fill_recipe_trigger(&render, 19),
        trigger_envelope(&render),
        19,
    );
    for _ in 0..frames_per_step {
        let _ = voices.render_sample(&render, sample_rate);
    }
    voices.trigger(
        test_fill_recipe_trigger(&render, 20),
        trigger_envelope(&render),
        20,
    );

    let mut kick = Vec::with_capacity(1_024);
    let mut snare = Vec::with_capacity(1_024);
    let mut hat = Vec::with_capacity(1_024);
    for _ in 0..1_024 {
        let sample = voices.render_sample(&render, sample_rate);
        kick.push(sample.kick);
        snare.push(sample.snare);
        hat.push(sample.hat);
    }

    let kick_rms = test_signal_rms(&kick);
    let snare_rms = test_signal_rms(&snare);
    let hat_rms = test_signal_rms(&hat);
    assert!(kick_rms > 0.10, "kick tail was truncated: {kick_rms}");
    assert!(snare_rms > 0.04, "snare tail was truncated: {snare_rms}");
    assert!(
        hat_rms > 0.001,
        "the setup hat tail did not survive into the following snare/kick call: {hat_rms}"
    );
}

#[test]
fn phrase_drive_signature_chokes_cleanly_and_exposes_a_pitch_dive_with_flam() {
    let render = cursor_20_phrase_drive_fill_render();
    let sample_rate = 48_000;
    let envelope = trigger_envelope(&render);

    let mut choked_voices = tr909_fill_voice::Tr909FillVoiceState::default();
    choked_voices.start();
    choked_voices.trigger(test_fill_recipe_trigger(&render, 23), envelope, 23);
    for _ in 0..128 {
        let _ = choked_voices.render_sample(&render, sample_rate);
    }
    choked_voices.choke();
    let choked = (0..768)
        .map(|_| {
            let sample = choked_voices.render_sample(&render, sample_rate);
            sample.kick + sample.snare + sample.hat
        })
        .collect::<Vec<_>>();

    assert!(
        test_signal_rms(&choked[..96]) > 0.01,
        "choke fixture had no tail to remove"
    );
    assert!(
        choked[384..].iter().all(|sample| *sample == 0.0),
        "six-millisecond choke left a hidden audible tail"
    );
    assert!(
        (choked[288] - choked[287]).abs() < 0.02,
        "choke ended with a click-sized discontinuity"
    );

    let mut stomp_voices = tr909_fill_voice::Tr909FillVoiceState::default();
    stomp_voices.start();
    stomp_voices.trigger_dive_stomp(test_fill_recipe_trigger(&render, 30), envelope, 30);
    let samples = (0..4_096)
        .map(|_| stomp_voices.render_sample(&render, sample_rate))
        .collect::<Vec<_>>();
    let kick = samples.iter().map(|sample| sample.kick).collect::<Vec<_>>();
    let snare = samples
        .iter()
        .map(|sample| sample.snare)
        .collect::<Vec<_>>();
    let early_kick_high = test_band_rms(&kick[..720], 120.0, 300.0, sample_rate, 1);
    let late_kick_low = test_band_rms(&kick[1_200..2_640], 40.0, 90.0, sample_rate, 1);
    let late_kick_high = test_band_rms(&kick[1_200..2_640], 120.0, 300.0, sample_rate, 1);
    let first_crack = test_signal_rms(&snare[..96]);
    let pre_flam = test_signal_rms(&snare[432..528]);
    let second_crack = test_signal_rms(&snare[528..624]);

    eprintln!(
        "signature stomp early_high={early_kick_high:.6} late_low={late_kick_low:.6} late_high={late_kick_high:.6} first_crack={first_crack:.6} pre_flam={pre_flam:.6} second_crack={second_crack:.6}"
    );
    assert!(
        early_kick_high > late_kick_high * 1.35,
        "DiveStomp kick did not fall out of its high opening pitch"
    );
    assert!(
        late_kick_low > late_kick_high * 1.25,
        "DiveStomp kick did not settle into low drum body"
    );
    assert!(
        second_crack > pre_flam * 1.20 && second_crack > first_crack * 0.35,
        "delayed snare flam was not a distinct subordinate second crack"
    );
}

#[test]
fn fill_voices_have_distinct_body_and_attack_regions() {
    let render = cursor_20_phrase_drive_fill_render();
    let sample_rate = 48_000;
    let kick = render_isolated_fill_voice_component(&render, 16, sample_rate, 4_096, 0);
    let snare = render_isolated_fill_voice_component(&render, 20, sample_rate, 4_096, 1);
    let hat = render_isolated_fill_voice_component(&render, 18, sample_rate, 4_096, 2);

    let kick_low = test_band_rms(&kick, 40.0, 120.0, sample_rate, 1);
    let snare_body = test_band_rms(&snare, 120.0, 500.0, sample_rate, 1);
    let hat_attack = test_band_rms(&hat, 2_000.0, 10_000.0, sample_rate, 1);
    let kick_attack = test_band_rms(&kick, 2_000.0, 10_000.0, sample_rate, 1);
    let hat_body = test_band_rms(&hat, 120.0, 500.0, sample_rate, 1);

    assert!(
        kick_low > 0.04,
        "kick had no absolute low drum body: {kick_low}"
    );
    assert!(
        snare_body > 0.04,
        "snare had no absolute 120-500 Hz body: {snare_body}"
    );
    assert!(
        hat_attack > 0.02,
        "hat had no absolute 2-10 kHz attack: {hat_attack}"
    );
    assert!(
        kick_low > kick_attack * 2.0,
        "kick collapsed into click energy"
    );
    assert!(
        hat_attack > hat_body * 1.5,
        "hat collapsed into low-mid body"
    );
}

#[test]
fn phrase_drive_fill_adds_body_and_transient_shape_over_candidate6_composite_control() {
    let render_state = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        ..Tr909RenderState::default()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let bar_frames = (sample_rate as f32 * 60.0 / render_state.tempo_bpm * 4.0).round() as usize;
    let candidate = render_tr909_offline(&render_state, sample_rate, channel_count, bar_frames);
    let composite =
        render_candidate6_composite_control(&render_state, sample_rate, channel_count, bar_frames);
    let last_beat_start = candidate.len() * 3 / 4;
    let candidate_last = &candidate[last_beat_start..];
    let composite_last = &composite[last_beat_start..];
    let candidate_kick = test_band_rms(candidate_last, 40.0, 120.0, sample_rate, 2);
    let candidate_body = test_band_rms(candidate_last, 120.0, 500.0, sample_rate, 2);
    let candidate_attack = test_band_rms(candidate_last, 2_000.0, 10_000.0, sample_rate, 2);
    let composite_body = test_band_rms(composite_last, 120.0, 500.0, sample_rate, 2);
    let composite_attack = test_band_rms(composite_last, 2_000.0, 10_000.0, sample_rate, 2);
    let candidate_metrics = signal_metrics(&candidate);
    let last_metrics = signal_metrics(candidate_last);
    let max_frame_delta = test_max_frame_delta(candidate_last, 2);
    let slot_sample_count = candidate_last.len() / 8;
    let slot_rms = (0..8)
        .map(|slot| {
            test_signal_rms(
                &candidate_last[slot * slot_sample_count..(slot + 1) * slot_sample_count],
            )
        })
        .collect::<Vec<_>>();

    eprintln!(
        "candidate bands low={candidate_kick:.6} body={candidate_body:.6} attack={candidate_attack:.6}; composite body={composite_body:.6} attack={composite_attack:.6}; slots={slot_rms:?}; candidate={candidate_metrics:?} last={last_metrics:?} frame_delta={max_frame_delta:.6}"
    );
    assert!(
        candidate_kick > 0.006,
        "fill had no absolute 40-120 Hz drum body"
    );
    assert!(
        candidate_body > 0.012,
        "fill had no absolute 120-500 Hz body"
    );
    assert!(
        candidate_attack > 0.006,
        "fill had no absolute 2-10 kHz attack"
    );
    assert!(
        candidate_body > composite_body * 1.20,
        "independent voices did not improve the exposed drum body: candidate={candidate_body:.6} control={composite_body:.6}"
    );
    assert!(
        candidate_attack > composite_attack * 1.05,
        "independent voices did not improve transient articulation: candidate={candidate_attack:.6} control={composite_attack:.6}"
    );
    assert!(
        last_metrics.peak_abs > 0.20,
        "final beat lacked a decisive drum transient"
    );
    assert_eq!(
        candidate_metrics.clip_count, 0,
        "fill clipped before the master bus"
    );
    assert!(
        candidate_metrics.peak_abs < master_bus_limiter_threshold(),
        "fill relied on the master limiter for headroom: {candidate_metrics:?}"
    );
    let choke_window_max = slot_rms[4..6].iter().copied().fold(0.0_f32, f32::max);
    assert!(
        slot_rms[6] > slot_rms[0] * 1.05
            && slot_rms[6] > choke_window_max * 3.0
            && slot_rms[6] > slot_rms[7] * 1.05,
        "late DiveStomp did not dominate the setup, extended choke, and tail: {slot_rms:?}"
    );
    assert!(
        max_frame_delta < 0.70,
        "fill introduced a click-sized discontinuity: {max_frame_delta}"
    );
}
