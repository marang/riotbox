#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(buffer: &[f32]) -> (usize, f32, f32) {
        let active = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let rms =
            (buffer.iter().map(|sample| sample * sample).sum::<f32>() / buffer.len() as f32).sqrt();
        (active, peak, rms)
    }

    #[test]
    fn follower_and_answer_shapes_are_audible_and_distinct() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut answer = vec![0.0; 44_100 * 2];

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                routing: Mc202RenderRouting::MusicBusBass,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                touch: 0.62,
                is_transport_running: true,
                ..Mc202RenderState::default()
            },
        );
        render_mc202_buffer(
            &mut answer,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Answer,
                routing: Mc202RenderRouting::MusicBusBass,
                phrase_shape: Mc202PhraseShape::AnswerHook,
                touch: 0.78,
                is_transport_running: true,
                ..Mc202RenderState::default()
            },
        );

        let follower_metrics = metrics(&follower);
        let answer_metrics = metrics(&answer);

        assert!(follower_metrics.0 > 10_000);
        assert!(answer_metrics.0 > 10_000);
        assert!((follower_metrics.2 - answer_metrics.2).abs() > 0.001);
    }

    #[test]
    fn touch_changes_render_energy_on_same_phrase() {
        let mut low_touch = vec![0.0; 44_100 * 2];
        let mut high_touch = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut low_touch,
            44_100,
            2,
            &Mc202RenderState {
                touch: 0.08,
                ..base
            },
        );
        render_mc202_buffer(
            &mut high_touch,
            44_100,
            2,
            &Mc202RenderState {
                touch: 0.92,
                ..base
            },
        );

        let low_metrics = metrics(&low_touch);
        let high_metrics = metrics(&high_touch);
        let max_delta = low_touch
            .iter()
            .zip(high_touch.iter())
            .map(|(low, high)| (low - high).abs())
            .fold(0.0_f32, f32::max);

        assert!(low_metrics.0 > 10_000);
        assert!(high_metrics.0 > 10_000);
        assert!(
            high_metrics.2 > low_metrics.2 + 0.006,
            "low RMS {:.6}, high RMS {:.6}",
            low_metrics.2,
            high_metrics.2
        );
        assert!(max_delta > 0.02, "max touch delta {max_delta}");
    }

    #[test]
    fn mutated_phrase_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut mutated = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut mutated,
            44_100,
            2,
            &Mc202RenderState {
                phrase_shape: Mc202PhraseShape::MutatedDrive,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let mutated_metrics = metrics(&mutated);
        let delta_rms = (follower
            .iter()
            .zip(mutated.iter())
            .map(|(follower, mutated)| (follower - mutated).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(mutated.iter())
            .map(|(follower, mutated)| (follower - mutated).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(mutated_metrics.0 > 10_000);
        assert!(delta_rms > 0.005, "mutated phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.02, "mutated phrase max delta {max_delta}");
    }

    #[test]
    fn pressure_cell_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut pressure = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.84,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut pressure,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Pressure,
                phrase_shape: Mc202PhraseShape::PressureCell,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let pressure_metrics = metrics(&pressure);
        let delta_rms = (follower
            .iter()
            .zip(pressure.iter())
            .map(|(follower, pressure)| (follower - pressure).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(pressure.iter())
            .map(|(follower, pressure)| (follower - pressure).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(pressure_metrics.0 > 10_000);
        assert!(delta_rms > 0.004, "pressure phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.02, "pressure phrase max delta {max_delta}");
    }

    #[test]
    fn note_budget_reduces_density_without_silencing_phrase() {
        let mut wide = vec![0.0; 44_100 * 2 * 2];
        let mut balanced = vec![0.0; 44_100 * 2 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut wide,
            44_100,
            2,
            &Mc202RenderState {
                note_budget: Mc202NoteBudget::Wide,
                ..base
            },
        );
        render_mc202_buffer(
            &mut balanced,
            44_100,
            2,
            &Mc202RenderState {
                note_budget: Mc202NoteBudget::Balanced,
                ..base
            },
        );

        let wide_metrics = metrics(&wide);
        let balanced_metrics = metrics(&balanced);
        let delta_rms = (wide
            .iter()
            .zip(balanced.iter())
            .map(|(wide, balanced)| (wide - balanced).powi(2))
            .sum::<f32>()
            / wide.len() as f32)
            .sqrt();

        assert!(wide_metrics.0 > 10_000);
        assert!(balanced_metrics.0 > 10_000);
        assert!(
            balanced_metrics.2 < wide_metrics.2,
            "balanced RMS {} should stay below wide RMS {}",
            balanced_metrics.2,
            wide_metrics.2
        );
        assert!(delta_rms > 0.001, "note-budget delta RMS {delta_rms}");
    }

    #[test]
    fn contour_hint_changes_phrase_without_silencing_it() {
        let mut neutral = vec![0.0; 44_100 * 2];
        let mut lift = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(&mut neutral, 44_100, 2, &base);
        render_mc202_buffer(
            &mut lift,
            44_100,
            2,
            &Mc202RenderState {
                contour_hint: Mc202ContourHint::Lift,
                ..base
            },
        );

        let neutral_metrics = metrics(&neutral);
        let lift_metrics = metrics(&lift);
        let delta_rms = (neutral
            .iter()
            .zip(lift.iter())
            .map(|(neutral, lift)| (neutral - lift).powi(2))
            .sum::<f32>()
            / neutral.len() as f32)
            .sqrt();

        assert!(neutral_metrics.0 > 10_000);
        assert!(lift_metrics.0 > 10_000);
        assert!(delta_rms > 0.004, "contour hint delta RMS {delta_rms}");
    }

    #[test]
    fn hook_response_leaves_space_without_silencing_phrase() {
        let mut direct = vec![0.0; 44_100 * 2 * 2];
        let mut response = vec![0.0; 44_100 * 2 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Neutral,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(&mut direct, 44_100, 2, &base);
        render_mc202_buffer(
            &mut response,
            44_100,
            2,
            &Mc202RenderState {
                hook_response: Mc202HookResponse::AnswerSpace,
                ..base
            },
        );

        let direct_metrics = metrics(&direct);
        let response_metrics = metrics(&response);
        let delta_rms = (direct
            .iter()
            .zip(response.iter())
            .map(|(direct, response)| (direct - response).powi(2))
            .sum::<f32>()
            / direct.len() as f32)
            .sqrt();

        assert!(direct_metrics.0 > 10_000);
        assert!(response_metrics.0 > 5_000);
        assert!(response_metrics.2 < direct_metrics.2);
        assert!(delta_rms > 0.004, "hook-response delta RMS {delta_rms}");
    }

    #[test]
    fn instigator_spike_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut instigator = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.90,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut instigator,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Instigator,
                phrase_shape: Mc202PhraseShape::InstigatorSpike,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let instigator_metrics = metrics(&instigator);
        let delta_rms = (follower
            .iter()
            .zip(instigator.iter())
            .map(|(follower, instigator)| (follower - instigator).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(instigator.iter())
            .map(|(follower, instigator)| (follower - instigator).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(instigator_metrics.0 > 8_000);
        assert!(delta_rms > 0.010, "instigator phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.04, "instigator phrase max delta {max_delta}");
    }

    #[test]
    fn render_is_stable_across_callback_chunk_boundaries() {
        let render = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };
        let mut whole = vec![0.0; 44_100 * 2];
        let mut chunked = vec![0.0; 44_100 * 2];
        let split_frames = 2_048;
        let split_samples = split_frames * 2;

        render_mc202_buffer(&mut whole, 44_100, 2, &render);
        render_mc202_buffer(&mut chunked[..split_samples], 44_100, 2, &render);

        let mut second_render = render;
        second_render.position_beats +=
            split_frames as f64 * f64::from(render.tempo_bpm) / 60.0 / 44_100.0;
        render_mc202_buffer(&mut chunked[split_samples..], 44_100, 2, &second_render);

        let max_delta = whole
            .iter()
            .zip(chunked.iter())
            .map(|(whole, chunked)| (whole - chunked).abs())
            .fold(0.0_f32, f32::max);
        assert!(max_delta < 0.0001, "max chunk boundary delta {max_delta}");
    }
}
