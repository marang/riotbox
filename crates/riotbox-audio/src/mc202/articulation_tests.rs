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

    fn low_band_rms(buffer: &[f32]) -> f32 {
        let mut low = 0.0_f32;
        let mut energy = 0.0_f32;
        let alpha = 0.018_f32;
        for sample in buffer {
            low += (*sample - low) * alpha;
            energy += low * low;
        }
        (energy / buffer.len() as f32).sqrt()
    }

    fn transient_energy(buffer: &[f32]) -> f32 {
        if buffer.len() < 2 {
            return 0.0;
        }
        let mut energy = 0.0_f32;
        for window in buffer.windows(2) {
            let delta = window[1] - window[0];
            energy += delta * delta;
        }
        (energy / (buffer.len() - 1) as f32).sqrt()
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

    #[test]
    fn source_phrase_production_sound_design_separates_body_from_bite_without_clipping() {
        let mut bass_pressure = vec![0.0; 44_100 * 2];
        let mut answer_stab = vec![0.0; 44_100 * 2];
        let scaffold = Mc202SourcePhraseRenderPlan {
            active_mask: 0b0001_0001_0001_0001,
            semitones: [-19, 0, 0, 0, -17, 0, 0, 0, -22, 0, 0, 0, -14, 0, 0, 0],
            accent_mask: 0b0001_0001_0001_0001,
            destructive_mask: 0b0000_0000_0001_0000,
            pressure: 0.72,
            contrast: 0.62,
            bass_weight: 0.0,
            stab_bite: 0.0,
            gate_snap: 0.0,
        };
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.84,
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
                    stab_bite: 0.08,
                    gate_snap: 0.12,
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
                    bass_weight: 0.10,
                    stab_bite: 0.96,
                    gate_snap: 0.90,
                    ..scaffold
                }),
                ..base
            },
        );

        let bass_metrics = metrics(&bass_pressure);
        let stab_metrics = metrics(&answer_stab);
        let bass_low = low_band_rms(&bass_pressure);
        let stab_low = low_band_rms(&answer_stab);
        let bass_transient = transient_energy(&bass_pressure);
        let stab_transient = transient_energy(&answer_stab);
        let bass_transient_sharpness = bass_transient / bass_metrics.2.max(f32::EPSILON);
        let stab_transient_sharpness = stab_transient / stab_metrics.2.max(f32::EPSILON);

        assert!(bass_metrics.2 > 0.001, "bass should remain audible: {bass_metrics:?}");
        assert!(stab_metrics.2 > 0.001, "stab should remain audible: {stab_metrics:?}");
        assert!(bass_metrics.1 <= 0.985, "bass clipped: {bass_metrics:?}");
        assert!(stab_metrics.1 <= 0.985, "stab clipped: {stab_metrics:?}");
        assert!(
            bass_low > stab_low * 1.30,
            "pressure body did not exceed stab low-band enough: bass_low={bass_low:.6} stab_low={stab_low:.6}"
        );
        assert!(
            stab_transient_sharpness > bass_transient_sharpness * 1.10,
            "answer stab did not exceed pressure transient sharpness enough: bass_sharpness={bass_transient_sharpness:.6} stab_sharpness={stab_transient_sharpness:.6}"
        );
    }
}
