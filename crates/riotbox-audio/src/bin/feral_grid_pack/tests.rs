#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_audio::runtime::signal_metrics;

    #[test]
    fn parses_grid_controls() {
        let parsed = Args::parse([
            "--source".to_string(),
            "input.wav".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
            "--bpm".to_string(),
            "130.0".to_string(),
            "--bars".to_string(),
            "4".to_string(),
            "--source-window-seconds".to_string(),
            "0.5".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
        assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
        assert_eq!(parsed.bpm, 130.0);
        assert!(parsed.bpm_overridden);
        assert_eq!(parsed.bars, 4);
        assert_eq!(parsed.source_window_seconds, 0.5);
    }

    #[test]
    fn rejects_missing_source() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn rejects_single_bar_pack() {
        assert!(
            Args::parse([
                "--source".to_string(),
                "input.wav".to_string(),
                "--bars".to_string(),
                "1".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn grid_uses_cumulative_frame_rounding() {
        let grid = Grid::new(128.0, 4, 8).expect("grid");

        assert_eq!(grid.total_beats, 32);
        assert_eq!(grid.total_frames, frames_for_beats(128.0, 32));
        assert_eq!(grid.bar_frame_count(0), grid.bar_end_frame(0));
        assert_eq!(grid.bar_end_frame(7), grid.total_frames);
    }

    #[test]
    fn bar_variation_metrics_distinguish_identical_and_alternating_bars() {
        let grid = Grid::new(120.0, 4, 3).expect("grid");
        let identical = bar_pattern_samples(&grid, |_, frame, bar_frames| frame < bar_frames / 8);
        let alternating = bar_pattern_samples(&grid, |bar, frame, bar_frames| {
            if bar.is_multiple_of(2) {
                frame < bar_frames / 8
            } else {
                frame > bar_frames / 2 && frame < bar_frames / 2 + bar_frames / 8
            }
        });

        let identical_metrics = bar_variation_metrics(&identical, &grid);
        let alternating_metrics = bar_variation_metrics(&alternating, &grid);

        assert!(identical_metrics.bar_similarity > 0.999);
        assert_eq!(identical_metrics.identical_bar_run_length, 3);
        assert!(alternating_metrics.bar_similarity < 0.25);
        assert_eq!(alternating_metrics.identical_bar_run_length, 1);
    }

    #[test]
    fn spectral_energy_metrics_distinguish_low_and_high_content() {
        let low = tone_samples(80.0, SAMPLE_RATE as usize / 2);
        let high = tone_samples(4_200.0, SAMPLE_RATE as usize / 2);

        let low_metrics = spectral_energy_metrics(&low);
        let high_metrics = spectral_energy_metrics(&high);

        assert!(low_metrics.low_band_energy_ratio > high_metrics.low_band_energy_ratio);
        assert!(high_metrics.high_band_energy_ratio > low_metrics.high_band_energy_ratio);
    }

    #[test]
    fn mix_balance_gate_rejects_old_drum_dominant_policy() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let sample_count = grid.total_frames * usize::from(CHANNEL_COUNT);
        let tr909 = vec![0.06; sample_count];
        let mc202 = vec![0.05; sample_count];
        let w30 = vec![0.20; sample_count];
        let old_drum_dominant_policy = MixPolicy {
            tr909_gain: 10.0,
            tr909_low_gain: 18.0,
            mc202_gain: 0.0,
            mc202_low_gain: 0.0,
            w30_gain: 0.94,
            drive: 1.7,
            output_gain: 0.92,
        };

        let old_ratio =
            generated_to_source_rms_ratio(&tr909, &mc202, &w30, &grid, old_drum_dominant_policy);
        let source_first_ratio =
            source_first_generated_to_source_rms_ratio(&tr909, &mc202, &w30, &grid);
        let support_ratio = support_generated_to_source_rms_ratio(&tr909, &mc202, &w30, &grid);

        assert!(old_ratio >= MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO);
        assert!(source_first_ratio < MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO);
        assert!(source_first_ratio <= 0.08);
        assert!(support_ratio >= 0.16);
        assert!(support_ratio < MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO);
    }

    #[test]
    fn all_lane_mix_movement_proof_rejects_collapsed_support_mix() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let tr909 = bar_pattern_samples(&grid, |_, frame, bar_frames| frame < bar_frames / 10);
        let mc202 =
            bar_pattern_samples(&grid, |_, frame, bar_frames| frame > bar_frames / 2 && frame < bar_frames / 2 + bar_frames / 12);
        let w30 = bar_pattern_samples(&grid, |bar, frame, bar_frames| {
            if bar.is_multiple_of(2) {
                frame < bar_frames / 6
            } else {
                frame > bar_frames / 3 && frame < bar_frames / 3 + bar_frames / 6
            }
        });
        let source_first = render_source_first_mix(&tr909, &mc202, &w30);
        let support = render_generated_support_mix(&tr909, &mc202, &w30);
        let proof = all_lane_mix_movement_proof(&tr909, &mc202, &w30, &source_first, &support, &grid);
        let collapsed = all_lane_mix_movement_proof(&tr909, &mc202, &w30, &support, &support, &grid);

        assert!(proof.applied, "{proof:?}");
        assert!(proof.source_first_to_support_rms_delta >= ALL_LANE_MIX_MIN_RMS_DELTA);
        assert!(proof.source_first_to_support_correlation <= ALL_LANE_MIX_MAX_CORRELATION);
        assert!(collapsed.source_first_to_support_rms_delta < ALL_LANE_MIX_MIN_RMS_DELTA);
        assert!(!collapsed.applied, "{collapsed:?}");
    }

    #[test]
    fn source_aware_tr909_profile_changes_for_same_bpm_sources() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let low_source = tone_samples(80.0, frames_for_beats(128.0, 8));
        let high_source = tone_samples(4_200.0, frames_for_beats(128.0, 8));

        let low_profile = derive_source_aware_tr909_profile(&low_source, &grid);
        let high_profile = derive_source_aware_tr909_profile(&high_source, &grid);
        let low_render = render_tr909_source_support(&grid, low_profile);
        let high_render = render_tr909_source_support(&grid, high_profile);
        let low_render_repeat = render_tr909_source_support(&grid, low_profile);
        let legacy_low_render = render_tr909_source_support_legacy(&grid, low_profile);
        let (_, kick_pressure, tr909_accent_dynamics) =
            render_tr909_source_support_with_pressure_and_accents(&grid, low_profile);
        let source_contour = Mc202SourceContourProfile::from_source_window(&low_source, &grid);
        let (mc202_low_render, mc202_pressure, mc202_source_contour) =
            render_mc202_bass_pressure_with_source_contour(&grid, low_profile, source_contour);

        assert_eq!(low_profile.support_profile, Tr909SourceSupportProfile::DropDrive);
        assert_eq!(high_profile.support_profile, Tr909SourceSupportProfile::BreakLift);
        assert_ne!(low_profile.pattern_adoption, high_profile.pattern_adoption);
        assert_ne!(low_render, high_render);
        assert_eq!(low_render, low_render_repeat);
        assert_ne!(low_render, legacy_low_render);
        assert!(kick_pressure.applied, "{kick_pressure:?}");
        assert!(kick_pressure.low_band_rms_ratio >= TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO);
        assert!(tr909_accent_dynamics.applied, "{tr909_accent_dynamics:?}");
        assert!(
            tr909_accent_dynamics.distinct_accent_count
                >= TR909_SOURCE_ACCENT_MIN_DISTINCT_ACCENTS,
            "{tr909_accent_dynamics:?}"
        );
        assert!(
            tr909_accent_dynamics.accent_span >= TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN,
            "{tr909_accent_dynamics:?}"
        );
        assert!(mc202_pressure.applied, "{mc202_pressure:?}");
        assert!(
            mc202_pressure.signal_rms >= MC202_BASS_PRESSURE_MIN_SIGNAL_RMS,
            "{mc202_pressure:?}"
        );
        assert!(
            mc202_pressure.low_band_rms >= MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS,
            "{mc202_pressure:?}"
        );
        assert_eq!(
            mc202_pressure.pressure_role,
            "bass_pressure_with_source_contour"
        );
        assert!(
            mc202_pressure.pressure_reinforcement_gain > 0.0,
            "{mc202_pressure:?}"
        );
        assert!(
            mc202_pressure.low_to_mid_energy_ratio
                >= MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO,
            "{mc202_pressure:?}"
        );
        assert!(
            mc202_pressure.low_to_high_energy_ratio > mc202_pressure.low_to_mid_energy_ratio,
            "{mc202_pressure:?}"
        );
        assert!(mc202_pressure.phrase_variation_applied, "{mc202_pressure:?}");
        assert!(mc202_pressure.distinct_bar_profile_count >= 2);
        assert!(mc202_source_contour.applied, "{mc202_source_contour:?}");
        assert_eq!(mc202_source_contour.contour_hint, Mc202ContourHint::Drop);
        assert!(
            mc202_source_contour.source_contour_delta_rms
                >= MC202_SOURCE_CONTOUR_MIN_DELTA_RMS,
            "{mc202_source_contour:?}"
        );
        assert_eq!(
            mc202_pressure.reason,
            "mc202_source_grid_proof_renderer"
        );
        assert!(signal_metrics(&low_render).rms > MIN_SIGNAL_RMS);
        assert!(signal_metrics(&high_render).rms > MIN_SIGNAL_RMS);
        assert!(signal_metrics(&mc202_low_render).rms > MIN_SIGNAL_RMS);
        assert!(
            source_grid_output_drift_metrics(&mc202_low_render, &grid).hit_ratio
                >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        );
    }

    #[test]
    fn tonal_hold_contour_keeps_mc202_support_reviewable() {
        let grid = Grid::new(120.0, 4, 4).expect("grid");
        let source_contour = Mc202SourceContourProfile {
            contour_hint: Mc202ContourHint::Hold,
            note_budget: Mc202NoteBudget::Sparse,
            touch_boost: 0.035,
            music_bus_boost: 0.025,
            low_band_energy_ratio: 0.23,
            mid_band_energy_ratio: 0.73,
            high_band_energy_ratio: 0.04,
            event_density_per_bar: 0.5,
            reason: "source_mid_section_hold_contour",
        };
        let tr909_profile = SourceAwareTr909Profile {
            signal_rms: 0.10,
            low_band_rms: 0.04,
            onset_count: 4,
            event_density_per_bar: 0.5,
            low_band_energy_ratio: 0.23,
            mid_band_energy_ratio: 0.73,
            high_band_energy_ratio: 0.04,
            support_profile: Tr909SourceSupportProfile::SteadyPulse,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::SupportPulse,
            phrase_variation: Tr909PhraseVariation::PhraseAnchor,
            drum_bus_level: 0.70,
            slam_intensity: 0.16,
            reason: "source_steady_pulse",
        };

        let (mc202_render, mc202_pressure, mc202_source_contour) =
            render_mc202_bass_pressure_with_source_contour(&grid, tr909_profile, source_contour);

        assert!(mc202_pressure.applied, "{mc202_pressure:?}");
        assert_eq!(mc202_pressure.low_body_emphasis, 0.0);
        assert!(
            mc202_pressure.pressure_reinforcement_gain >= 0.038,
            "{mc202_pressure:?}"
        );
        assert!(
            mc202_pressure.signal_rms >= 0.0055,
            "{mc202_pressure:?}"
        );
        assert!(mc202_source_contour.applied, "{mc202_source_contour:?}");
        assert!(signal_metrics(&mc202_render).rms >= 0.0055);
    }

    #[test]
    fn low_dominant_drop_contour_gets_physical_mc202_body_without_affecting_hold() {
        let grid = Grid::new(120.0, 4, 4).expect("grid");
        let drop_contour = Mc202SourceContourProfile {
            contour_hint: Mc202ContourHint::Drop,
            note_budget: Mc202NoteBudget::Balanced,
            touch_boost: 0.055,
            music_bus_boost: 0.040,
            low_band_energy_ratio: 0.86,
            mid_band_energy_ratio: 0.12,
            high_band_energy_ratio: 0.02,
            event_density_per_bar: 1.0,
            reason: "source_low_section_drop_contour",
        };
        let hold_contour = Mc202SourceContourProfile {
            contour_hint: Mc202ContourHint::Hold,
            note_budget: Mc202NoteBudget::Sparse,
            touch_boost: 0.035,
            music_bus_boost: 0.025,
            low_band_energy_ratio: 0.24,
            mid_band_energy_ratio: 0.72,
            high_band_energy_ratio: 0.04,
            event_density_per_bar: 0.5,
            reason: "source_mid_section_hold_contour",
        };
        let drop_profile = SourceAwareTr909Profile {
            signal_rms: 0.16,
            low_band_rms: 0.14,
            onset_count: 8,
            event_density_per_bar: 1.0,
            low_band_energy_ratio: 0.86,
            mid_band_energy_ratio: 0.12,
            high_band_energy_ratio: 0.02,
            support_profile: Tr909SourceSupportProfile::DropDrive,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::MainlineDrive,
            phrase_variation: Tr909PhraseVariation::PhraseDrive,
            drum_bus_level: 0.84,
            slam_intensity: 0.22,
            reason: "source_low_drive",
        };
        let hold_profile = SourceAwareTr909Profile {
            signal_rms: 0.10,
            low_band_rms: 0.04,
            onset_count: 4,
            event_density_per_bar: 0.5,
            low_band_energy_ratio: 0.24,
            mid_band_energy_ratio: 0.72,
            high_band_energy_ratio: 0.04,
            support_profile: Tr909SourceSupportProfile::SteadyPulse,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::SupportPulse,
            phrase_variation: Tr909PhraseVariation::PhraseAnchor,
            drum_bus_level: 0.70,
            slam_intensity: 0.16,
            reason: "source_steady_pulse",
        };

        let (drop_render, drop_pressure, drop_source_contour) =
            render_mc202_bass_pressure_with_source_contour(&grid, drop_profile, drop_contour);
        let (hold_render, hold_pressure, hold_source_contour) =
            render_mc202_bass_pressure_with_source_contour(&grid, hold_profile, hold_contour);
        let drop_spectral = spectral_energy_metrics(&drop_render);
        let hold_spectral = spectral_energy_metrics(&hold_render);

        assert!(drop_pressure.applied, "{drop_pressure:?}");
        assert!(hold_pressure.applied, "{hold_pressure:?}");
        assert!(drop_source_contour.applied, "{drop_source_contour:?}");
        assert!(hold_source_contour.applied, "{hold_source_contour:?}");
        assert!(
            drop_pressure.low_body_emphasis >= 0.34,
            "{drop_pressure:?}"
        );
        assert_eq!(hold_pressure.low_body_emphasis, 0.0);
        assert!(
            drop_pressure.pressure_reinforcement_gain > hold_pressure.pressure_reinforcement_gain,
            "drop={drop_pressure:?} hold={hold_pressure:?}"
        );
        assert!(
            drop_pressure.low_band_rms > hold_pressure.low_band_rms * 1.45,
            "drop={drop_pressure:?} hold={hold_pressure:?}"
        );
        assert!(
            drop_spectral.low_band_energy_ratio > hold_spectral.low_band_energy_ratio,
            "drop={drop_spectral:?} hold={hold_spectral:?}"
        );
        assert!(
            drop_pressure.peak_abs < 0.12,
            "{drop_pressure:?}"
        );
    }

    #[test]
    fn renders_grid_pack_files_and_noncollapsed_audio() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source_path = temp.path().join("source.wav");
        let output_dir = temp.path().join("pack");
        write_interleaved_pcm16_wav(
            &source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &synthetic_break_source(frames_for_beats(128.0, 8)),
        )
        .expect("write source");

        let args = Args {
            source_path,
            output_dir: Some(output_dir.clone()),
            date: "test".into(),
            bpm: 128.0,
            bpm_overridden: true,
            bars: 2,
            source_start_seconds: 0.0,
            source_window_seconds: 0.5,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(output_dir.join("stems/01_tr909_beat_fill.wav").is_file());
        assert!(
            output_dir
                .join("stems/02_w30_feral_source_chop.wav")
                .is_file()
        );
        assert!(output_dir.join("stems/03_mc202_bass_pressure.wav").is_file());
        assert!(output_dir.join("04_riotbox_source_first_mix.wav").is_file());
        assert!(
            output_dir
                .join("05_riotbox_generated_support_mix.wav")
                .is_file()
        );
        assert!(output_dir.join("grid-report.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let source_first_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("04_riotbox_source_first_mix.wav"))
                .expect("load full mix");
        let full_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("05_riotbox_generated_support_mix.wav"))
                .expect("load generated-support mix");
        let grid = Grid::new(128.0, 4, 2).expect("grid");

        assert_eq!(source_first_mix.frame_count(), grid.total_frames);
        assert_eq!(full_mix.frame_count(), grid.total_frames);
        assert!(signal_metrics(source_first_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
        assert!(signal_metrics(full_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
        assert!(
            signal_metrics(&one_pole_lowpass(full_mix.interleaved_samples(), 165.0)).rms
                > MIN_LOW_BAND_RMS
        );

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        super::manifest_assertions::assert_manifest_smoke_gate(&manifest, &output_dir);

        let report = fs::read_to_string(output_dir.join("grid-report.md")).expect("report");
        let readme = fs::read_to_string(output_dir.join("README.md")).expect("readme");
        for text in [&report, &readme] {
            assert!(text.contains("Source timing readiness: `"));
            assert!(text.contains("Source timing downbeat: `"));
            assert!(text.contains("Source timing phrase: `"));
            assert!(text.contains("phrases="));
            assert!(text.contains("bars="));
            assert!(text.contains("confidence="));
            assert!(text.contains("drift="));
            assert!(text.contains("Source timing warnings: `"));
        }
        assert!(report.contains("Source timing BPM: `primary="));
    }

    fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let bar_pulse = frame % frames_for_beats(128.0, 1);
            let kick = if bar_pulse < 1_200 {
                ((1.0 - bar_pulse as f32 / 1_200.0).max(0.0) * 0.9)
                    * (phase * 74.0 * std::f32::consts::TAU).sin()
            } else {
                0.0
            };
            let grit = (phase * 510.0 * std::f32::consts::TAU).sin() * 0.08;
            let sample = kick + grit;
            samples.push(sample);
            samples.push(sample * 0.97);
        }
        samples
    }

    fn bar_pattern_samples(
        grid: &Grid,
        is_active_frame: impl Fn(u32, usize, usize) -> bool,
    ) -> Vec<f32> {
        let mut samples = Vec::with_capacity(grid.total_frames * usize::from(CHANNEL_COUNT));
        for bar in 0..grid.bars {
            let bar_frames = grid.bar_frame_count(bar);
            for frame in 0..bar_frames {
                let sample = if is_active_frame(bar, frame, bar_frames) {
                    0.5
                } else {
                    0.0
                };
                samples.push(sample);
                samples.push(sample);
            }
        }
        samples
    }

    fn tone_samples(frequency_hz: f32, frame_count: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let sample = (phase * frequency_hz * std::f32::consts::TAU).sin() * 0.5;
            samples.push(sample);
            samples.push(sample);
        }
        samples
    }
}
