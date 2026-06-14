#[cfg(test)]
mod mc202_phrase_feature_tests {
    use super::*;

    #[test]
    fn mc202_source_phrase_features_are_deterministic_for_same_source_evidence() {
        let graph = mc202_feature_graph();
        let phrase = graph.timing.phrase_grid[0];

        let first = mc202_source_phrase_feature_vector(&graph, &phrase);
        let second = mc202_source_phrase_feature_vector(&graph, &phrase);

        assert_eq!(first, second);
        assert!(first.has_musical_evidence());
        assert!(first.low_band_pressure > 0.7);
        assert!(first.transient_density > 0.5);
        assert!(
            first
                .provenance_refs
                .iter()
                .any(|reference| reference.starts_with("anchor:kick-"))
        );
    }

    #[test]
    fn mc202_source_phrase_features_change_with_source_evidence() {
        let mut pressure_graph = mc202_feature_graph();
        let mut hook_graph = mc202_feature_graph();
        hook_graph.sections[0].label_hint = SectionLabelHint::Chorus;
        hook_graph.sections[0].energy_class = EnergyClass::Medium;
        hook_graph.sections[0].tags = vec!["hook".into(), "vocal".into()];
        hook_graph.assets[0].asset_type = AssetType::HookFragment;
        hook_graph.assets[0].tags = vec!["hook".into(), "lead".into()];
        hook_graph.candidates[0].candidate_type = CandidateType::HookCandidate;
        hook_graph.candidates[0].tags = vec!["hook".into()];
        hook_graph.timing.hypotheses[0].anchors.clear();
        hook_graph.analysis_summary.hook_candidate_count = 2;
        hook_graph.analysis_summary.overall_confidence = 0.72;
        pressure_graph.analysis_summary.hook_candidate_count = 0;

        let pressure = mc202_source_phrase_feature_vector(
            &pressure_graph,
            &pressure_graph.timing.phrase_grid[0],
        );
        let hook =
            mc202_source_phrase_feature_vector(&hook_graph, &hook_graph.timing.phrase_grid[0]);

        assert!(pressure.low_band_pressure > hook.low_band_pressure);
        assert!(pressure.transient_density > hook.transient_density);
        assert!(hook.hook_restraint > pressure.hook_restraint);
        assert_ne!(pressure, hook);
    }

    #[test]
    fn mc202_source_phrase_features_prefer_measured_audio_evidence() {
        let mut graph = mc202_feature_graph();
        graph.phrase_audio_features = vec![PhraseAudioFeatures {
            phrase_index: 2,
            start_seconds: 0.0,
            end_seconds: 16.0,
            start_bar: 8,
            end_bar: 15,
            low_band_rms: 0.34,
            low_mid_ratio: 0.76,
            low_band_movement: 0.88,
            transient_density: 0.22,
            offbeat_onset_density: 0.71,
            spectral_roughness: 0.41,
            spectral_brightness: 0.24,
            hook_restraint_hint: 0.19,
            confidence: 0.90,
            provenance_refs: vec!["mc202.phrase-audio-features.v0".into()],
        }];

        let features = mc202_source_phrase_feature_vector(&graph, &graph.timing.phrase_grid[0]);

        assert!(features.low_band_pressure > 0.85, "{features:?}");
        assert!(
            (features.transient_density - 0.22).abs() < 0.001,
            "{features:?}"
        );
        assert!(
            (features.offbeat_density - 0.71).abs() < 0.001,
            "{features:?}"
        );
        assert!(
            features
                .provenance_refs
                .iter()
                .any(|reference| reference
                    == "phrase_audio:mc202.phrase-audio-features.v0")
        );
        assert!(features.has_musical_evidence());
    }

    #[test]
    fn mc202_source_phrase_features_reject_empty_static_evidence() {
        let mut graph = minimal_source_graph();
        graph.timing.phrase_grid = vec![PhraseSpan {
            phrase_index: 1,
            start_bar: 8,
            end_bar: 15,
            confidence: 0.18,
        }];
        graph.timing.bpm_confidence = 0.12;

        let features = mc202_source_phrase_feature_vector(&graph, &graph.timing.phrase_grid[0]);

        assert!(features.stay_out);
        assert!(!features.has_musical_evidence());
        assert!(features.source_strength < 0.25);
        assert!(features.provenance_refs.is_empty());
    }

    fn mc202_feature_graph() -> SourceGraph {
        let mut graph = minimal_source_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-drop"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 8,
            bar_end: 15,
            energy_class: EnergyClass::Peak,
            confidence: 0.91,
            tags: vec!["bass_pressure".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-drum"),
            asset_type: AssetType::DrumAnchor,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 8,
            end_bar: 9,
            confidence: 0.88,
            tags: vec!["bass".into(), "pressure".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("candidate-kick"),
            candidate_type: CandidateType::KickAnchor,
            asset_ref: AssetId::from("asset-drum"),
            score: 0.92,
            confidence: 0.90,
            tags: vec!["low".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["fixture:mc202".into()],
        });
        graph.timing.bpm_estimate = Some(128.0);
        graph.timing.bpm_confidence = 0.86;
        graph.timing.meter_hint = Some(MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        });
        graph.timing.phrase_grid = vec![PhraseSpan {
            phrase_index: 2,
            start_bar: 8,
            end_bar: 15,
            confidence: 0.90,
        }];
        graph.timing.primary_hypothesis_id = Some("primary".into());
        graph.timing.hypotheses.push(TimingHypothesis {
            hypothesis_id: "primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 128.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.88,
            score: 0.91,
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: graph.timing.phrase_grid.clone(),
            anchors: vec![
                SourceTimingAnchor {
                    anchor_id: "kick-8".into(),
                    anchor_type: SourceTimingAnchorType::Kick,
                    time_seconds: 0.0,
                    bar_index: Some(8),
                    beat_index: Some(32),
                    confidence: 0.95,
                    strength: 0.94,
                    tags: vec!["low".into()],
                },
                SourceTimingAnchor {
                    anchor_id: "ghost-8".into(),
                    anchor_type: SourceTimingAnchorType::TransientCluster,
                    time_seconds: 0.3,
                    bar_index: Some(8),
                    beat_index: Some(33),
                    confidence: 0.72,
                    strength: 0.70,
                    tags: vec!["offbeat".into()],
                },
            ],
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: vec!["fixture:mc202".into()],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.89,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::High,
            loop_candidate_count: 0,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: Vec::new(),
        };
        graph
    }

    fn minimal_source_graph() -> SourceGraph {
        SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "break.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 180.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beats".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 42,
                run_notes: None,
            },
        )
    }
}
