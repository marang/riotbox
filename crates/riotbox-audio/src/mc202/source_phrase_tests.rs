#[cfg(test)]
mod source_phrase_tests {
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
    fn source_phrase_plan_renders_answer_without_primitive_answer_leak() {
        let mut primitive_answer = vec![0.0; 44_100 * 2];
        let mut source_answer = vec![0.0; 44_100 * 2];

        let primitive = Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.78,
            is_transport_running: true,
            ..Mc202RenderState::default()
        };
        let source_derived = Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                active_mask: 0b0010_0010_1000_0100,
                semitones: [0, 0, 0, 0, 0, 5, 0, 7, 0, 0, 3, 0, 0, 7, 0, 0],
                accent_mask: 0b0000_0000_1000_0100,
                destructive_mask: 0,
                pressure: 0.35,
                contrast: 0.60,
                bass_weight: 0.30,
                stab_bite: 0.60,
                gate_snap: 0.45,
            }),
            ..primitive
        };

        render_mc202_buffer(&mut primitive_answer, 44_100, 2, &primitive);
        render_mc202_buffer(&mut source_answer, 44_100, 2, &source_derived);

        let primitive_metrics = metrics(&primitive_answer);
        let source_metrics = metrics(&source_answer);

        assert_eq!(primitive_metrics.0, 0);
        assert!(source_metrics.0 > 5_000);
        assert!(source_metrics.2 > 0.001);
    }

    #[test]
    fn source_phrase_pressure_and_destructive_contrast_change_render_output() {
        let mut neutral = vec![0.0; 44_100 * 2];
        let mut pressured = vec![0.0; 44_100 * 2];
        let plan = Mc202SourcePhraseRenderPlan {
            active_mask: 0b0001_0001_0001_0001,
            semitones: [-12, 0, 0, 0, -10, 0, 0, 0, -15, 0, 0, 0, -7, 0, 0, 0],
            accent_mask: 0,
            destructive_mask: 0,
            pressure: 0.0,
            contrast: 0.0,
            bass_weight: 0.0,
            stab_bite: 0.0,
            gate_snap: 0.0,
        };
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.78,
            tempo_bpm: 128.0,
            is_transport_running: true,
            source_phrase_plan: Some(plan),
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(&mut neutral, 44_100, 2, &base);
        render_mc202_buffer(
            &mut pressured,
            44_100,
            2,
            &Mc202RenderState {
                source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                    accent_mask: plan.active_mask,
                    destructive_mask: 0b0000_0000_0001_0000,
                    pressure: 0.92,
                    contrast: 0.72,
                    bass_weight: 0.88,
                    stab_bite: 0.16,
                    gate_snap: 0.12,
                    ..plan
                }),
                ..base
            },
        );

        let neutral_metrics = metrics(&neutral);
        let pressure_metrics = metrics(&pressured);
        let max_delta = neutral
            .iter()
            .zip(pressured.iter())
            .map(|(left, right)| (left - right).abs())
            .fold(0.0_f32, f32::max);

        assert!(neutral_metrics.0 > 4_000);
        assert!(pressure_metrics.0 > 4_000);
        assert!(
            pressure_metrics.2 > neutral_metrics.2 * 1.20,
            "source pressure did not lift RMS enough: neutral={neutral_metrics:?} pressured={pressure_metrics:?}"
        );
        assert!(
            max_delta > 0.01,
            "source pressure/destructive contrast collapsed to neutral render: {max_delta}"
        );
    }
}
