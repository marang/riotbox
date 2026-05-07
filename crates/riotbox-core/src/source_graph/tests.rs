#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_candidates_by_type() {
        let mut graph = SourceGraph::new(
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
        );

        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("cand-1"),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-1"),
            score: 0.8,
            confidence: 0.9,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("cand-2"),
            candidate_type: CandidateType::HookCandidate,
            asset_ref: AssetId::from("asset-2"),
            score: 0.7,
            confidence: 0.8,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });

        assert_eq!(graph.loop_candidate_count(), 1);
        assert_eq!(graph.hook_candidate_count(), 1);
    }

    #[test]
    fn feral_break_support_evidence_requires_scorecard_relationship_and_hook_or_capture() {
        let mut graph = minimal_source_graph();
        graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::SupportsBreakRebuild,
            from_id: "asset-hook".into(),
            to_id: "section-break".into(),
            weight: 0.85,
            notes: Some("break can be rebuilt from hook".into()),
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-hook"),
            asset_type: AssetType::HookFragment,
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.9,
            tags: vec!["feral".into()],
            source_refs: vec!["src-1".into()],
        });

        assert!(graph.has_feral_break_support_evidence());

        graph.analysis_summary.break_rebuild_potential = QualityClass::Medium;
        assert!(!graph.has_feral_break_support_evidence());
    }

    #[test]
    fn feral_break_support_evidence_rejects_incomplete_scorecard() {
        let mut graph = minimal_source_graph();
        graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-hook"),
            asset_type: AssetType::HookFragment,
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.9,
            tags: vec!["feral".into()],
            source_refs: vec!["src-1".into()],
        });
        assert!(!graph.has_feral_break_support_evidence());

        graph.relationships.push(Relationship {
            relation_type: RelationshipType::SupportsBreakRebuild,
            from_id: "asset-hook".into(),
            to_id: "section-break".into(),
            weight: 0.49,
            notes: Some("weak relationship stays below Feral threshold".into()),
        });
        assert!(!graph.has_feral_break_support_evidence());

        graph.relationships[0].weight = 0.5;
        graph.assets.clear();
        graph.analysis_summary.hook_candidate_count = 0;
        assert!(!graph.has_feral_break_support_evidence());

        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("candidate-capture"),
            candidate_type: CandidateType::CaptureCandidate,
            asset_ref: AssetId::from("asset-hook"),
            score: 0.9,
            confidence: 0.85,
            tags: vec!["capture".into()],
            constraints: vec![],
            provenance_refs: vec!["provider:fixture".into()],
        });
        assert!(graph.has_feral_break_support_evidence());
    }

    #[test]
    fn source_graph_roundtrips_via_json() {
        let mut graph = SourceGraph::new(
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
                provider_set: vec!["beats".into(), "hooks".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 42,
                run_notes: Some("test".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["main".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-a"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec!["loop".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("candidate-a"),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-a"),
            score: 0.88,
            confidence: 0.91,
            tags: vec!["useful".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:beats".into()],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::BelongsToSection,
            from_id: "asset-a".into(),
            to_id: "section-a".into(),
            weight: 1.0,
            notes: Some("primary loop".into()),
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.87,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![AnalysisWarning {
                code: "low_hook_density".into(),
                message: "few hook fragments".into(),
            }],
        };

        let json = serde_json::to_string_pretty(&graph).expect("serialize source graph");
        let decoded: SourceGraph = serde_json::from_str(&json).expect("deserialize source graph");

        assert_eq!(decoded, graph);
    }

    #[test]
    fn rich_timing_hypotheses_roundtrip_and_select_primary_grid() {
        let mut graph = minimal_source_graph();
        graph.timing.bpm_estimate = Some(128.0);
        graph.timing.bpm_confidence = 0.72;
        graph.timing.meter_hint = Some(MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        });
        graph.timing.primary_hypothesis_id = Some("timing-primary".into());
        graph.timing.quality = TimingQuality::Medium;
        graph.timing.degraded_policy = TimingDegradedPolicy::Cautious;
        graph.timing.warnings.push(TimingWarning {
            code: TimingWarningCode::DoubleTimePossible,
            message: "double-time grid remains plausible".into(),
        });
        graph.timing.hypotheses.push(TimingHypothesis {
            hypothesis_id: "timing-primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 128.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.82,
            score: 0.91,
            beat_grid: vec![BeatPoint {
                beat_index: 0,
                time_seconds: 0.0,
                confidence: 0.95,
            }],
            bar_grid: vec![BarSpan {
                bar_index: 0,
                start_seconds: 0.0,
                end_seconds: 1.875,
                downbeat_confidence: 0.88,
                phrase_index: Some(0),
            }],
            phrase_grid: vec![PhraseSpan {
                phrase_index: 0,
                start_bar: 0,
                end_bar: 4,
                confidence: 0.75,
            }],
            anchors: vec![SourceTimingAnchor {
                anchor_id: "kick-0".into(),
                anchor_type: SourceTimingAnchorType::Kick,
                time_seconds: 0.0,
                bar_index: Some(0),
                beat_index: Some(0),
                confidence: 0.9,
                strength: 0.84,
                tags: vec!["downbeat".into()],
            }],
            drift: vec![TimingDriftReport {
                window_bars: 4,
                max_drift_ms: 12.0,
                mean_abs_drift_ms: 4.5,
                end_drift_ms: -2.0,
                confidence: 0.78,
            }],
            groove: vec![GrooveResidual {
                subdivision: GrooveSubdivision::Sixteenth,
                offset_ms: -8.0,
                confidence: 0.66,
            }],
            quality: TimingQuality::High,
            warnings: vec![TimingWarning {
                code: TimingWarningCode::WeakBackbeatAnchor,
                message: "backbeat is present but not dominant".into(),
            }],
            provenance: vec!["fixture:timing-grid".into()],
        });
        graph.timing.hypotheses.push(TimingHypothesis {
            hypothesis_id: "timing-double".into(),
            kind: TimingHypothesisKind::DoubleTime,
            bpm: 256.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.41,
            score: 0.4,
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: Vec::new(),
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::Low,
            warnings: vec![TimingWarning {
                code: TimingWarningCode::DoubleTimePossible,
                message: "alternate double-time path".into(),
            }],
            provenance: vec!["fixture:alternate".into()],
        });

        let json = serde_json::to_string_pretty(&graph).expect("serialize source graph");
        let decoded: SourceGraph = serde_json::from_str(&json).expect("deserialize source graph");

        assert_eq!(decoded, graph);
        assert_eq!(
            decoded
                .timing
                .primary_hypothesis()
                .map(|hypothesis| hypothesis.bpm),
            Some(128.0)
        );
        assert_eq!(
            decoded.timing.effective_timing_quality(),
            TimingQuality::Medium
        );
        assert_eq!(
            decoded.timing.effective_degraded_policy(),
            TimingDegradedPolicy::Cautious
        );
    }

    #[test]
    fn timing_model_loads_legacy_json_without_rich_timing_fields() {
        let timing: TimingModel = serde_json::from_str(
            r#"{
              "bpm_estimate": 126.0,
              "bpm_confidence": 0.61,
              "meter_hint": {"beats_per_bar": 4, "beat_unit": 4},
              "beat_grid": [],
              "bar_grid": [],
              "phrase_grid": []
            }"#,
        )
        .expect("deserialize legacy timing model");

        assert_eq!(timing.hypotheses, Vec::new());
        assert_eq!(timing.primary_hypothesis_id, None);
        assert_eq!(timing.quality, TimingQuality::Unknown);
        assert_eq!(timing.effective_timing_quality(), TimingQuality::Medium);
        assert_eq!(
            timing.effective_degraded_policy(),
            TimingDegradedPolicy::Cautious
        );
    }

    #[test]
    fn timing_model_without_analysis_disables_timing_aware_consumers() {
        let timing = TimingModel::default();

        assert_eq!(timing.effective_timing_quality(), TimingQuality::Unknown);
        assert_eq!(
            timing.effective_degraded_policy(),
            TimingDegradedPolicy::Disabled
        );
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
