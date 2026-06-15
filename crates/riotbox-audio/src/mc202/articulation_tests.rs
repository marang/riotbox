#[cfg(test)]
mod articulation_tests {
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
    fn source_phrase_articulation_separates_bass_pressure_from_answer_stab() {
        let mut bass_pressure = vec![0.0; 44_100 * 2];
        let mut answer_stab = vec![0.0; 44_100 * 2];
        let scaffold = Mc202SourcePhraseRenderPlan {
            active_mask: 0b0001_0001_0001_0001,
            semitones: [-12, 0, 0, 0, -10, 0, 0, 0, -15, 0, 0, 0, -7, 0, 0, 0],
            accent_mask: 0b0001_0001_0001_0001,
            destructive_mask: 0,
            pressure: 0.62,
            contrast: 0.55,
            bass_weight: 0.0,
            stab_bite: 0.0,
            gate_snap: 0.0,
        };
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.78,
            tempo_bpm: 128.0,
            is_transport_running: true,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut bass_pressure,
            44_100,
            2,
            &Mc202RenderState {
                source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                    bass_weight: 0.96,
                    stab_bite: 0.10,
                    gate_snap: 0.10,
                    ..scaffold
                }),
                ..base
            },
        );
        render_mc202_buffer(
            &mut answer_stab,
            44_100,
            2,
            &Mc202RenderState {
                source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                    bass_weight: 0.12,
                    stab_bite: 0.96,
                    gate_snap: 0.86,
                    ..scaffold
                }),
                ..base
            },
        );

        let bass_metrics = metrics(&bass_pressure);
        let stab_metrics = metrics(&answer_stab);
        let max_delta = bass_pressure
            .iter()
            .zip(answer_stab.iter())
            .map(|(left, right)| (left - right).abs())
            .fold(0.0_f32, f32::max);

        assert!(
            bass_metrics.0 > stab_metrics.0,
            "bass articulation should hold notes longer than stab articulation: bass={bass_metrics:?} stab={stab_metrics:?}"
        );
        assert!(bass_metrics.2 > 0.001, "{bass_metrics:?}");
        assert!(stab_metrics.2 > 0.001, "{stab_metrics:?}");
        assert!(
            max_delta > 0.01,
            "bass/stab articulation collapsed to the same render: {max_delta}"
        );
    }
}
