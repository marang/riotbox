#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{
        ids::{AssetId, SceneId, SectionId, SourceId},
        session::{Tr909LaneState, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
        source_graph::{
            Asset, AssetType, Candidate, CandidateType, DecodeProfile, EnergyClass,
            GraphProvenance, PhraseAudioFeatures, QualityClass, Relationship, RelationshipType,
            Section, SectionLabelHint, SourceDescriptor, SourceGraph,
        },
        transport::{
            DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState,
            TransportGridPosition,
        },
    };

    use super::*;

    #[derive(Debug, Deserialize)]
    struct RenderProjectionFixture {
        name: String,
        transport_position_beats: f64,
        #[serde(default)]
        scene_context: Option<String>,
        reinforcement_mode: Tr909ReinforcementModeState,
        takeover_enabled: bool,
        takeover_profile: Option<Tr909TakeoverProfileState>,
        pattern_ref: Option<String>,
        expected_mode: String,
        expected_routing: String,
        expected_pattern_adoption: Option<String>,
        expected_phrase_variation: Option<String>,
        expected_source_support_profile: Option<String>,
        expected_source_support_context: Option<String>,
        expected_takeover_profile: Option<String>,
    }

    fn sample_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 64.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-18T00:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("tr909-policy-fixture".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-drop"),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec!["drop".into()],
        });
        graph.sections.push(Section {
            section_id: SectionId::from("section-break"),
            label_hint: crate::source_graph::SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.85,
            tags: vec!["break".into()],
        });
        graph
    }

    fn steady_section_graph() -> SourceGraph {
        let mut graph = sample_graph();
        graph.sections.clear();
        graph.sections.push(Section {
            section_id: SectionId::from("section-steady"),
            label_hint: SectionLabelHint::Verse,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::Medium,
            confidence: 0.88,
            tags: vec!["steady".into()],
        });
        graph
    }

    fn seed_feral_break_support(graph: &mut SourceGraph) {
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-feral-hook"),
            asset_type: AssetType::HookFragment,
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.9,
            tags: vec!["feral".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: "candidate-feral-capture".into(),
            candidate_type: CandidateType::CaptureCandidate,
            asset_ref: AssetId::from("asset-feral-hook"),
            score: 0.9,
            confidence: 0.85,
            tags: vec!["feral".into()],
            constraints: vec!["capture_first".into()],
            provenance_refs: vec!["provider:fixture".into()],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::SupportsBreakRebuild,
            from_id: "asset-feral-hook".into(),
            to_id: "section-steady".into(),
            weight: 0.85,
            notes: Some("feral hook supports rebuild".into()),
        });
        graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        graph.analysis_summary.hook_candidate_count = 1;
    }

    fn transport_state(position_beats: f64) -> TransportClockState {
        let grid_position = TransportGridPosition::from_zero_based_position_beats(
            position_beats,
            DEFAULT_BEATS_PER_BAR,
            DEFAULT_BARS_PER_PHRASE,
        );
        TransportClockState {
            is_playing: true,
            position_beats,
            beat_index: grid_position.beat_cursor,
            bar_index: grid_position.bar_index,
            phrase_index: grid_position.phrase_index,
            current_scene: Some(SceneId::from("scene-1")),
        }
    }

    fn counter_rhythm_graph(
        transient_density: f32,
        offbeat_onset_density: f32,
        confidence: f32,
    ) -> SourceGraph {
        let mut graph = sample_graph();
        graph.phrase_audio_features.push(PhraseAudioFeatures {
            phrase_index: 1,
            start_seconds: 0.0,
            end_seconds: 8.0,
            start_bar: 1,
            end_bar: 4,
            low_band_rms: 0.1,
            low_mid_ratio: 0.4,
            low_band_movement: 0.2,
            transient_density,
            offbeat_onset_density,
            spectral_roughness: 0.3,
            spectral_brightness: 0.4,
            hook_restraint_hint: 0.2,
            confidence,
            provenance_refs: vec!["provider:synthetic-counter-rhythm-test".into()],
        });
        graph
    }

    fn break_reinforce_slam_lane() -> Tr909LaneState {
        Tr909LaneState {
            pattern_ref: Some("mainline-counter-rhythm".into()),
            takeover_enabled: false,
            takeover_profile: None,
            slam_enabled: true,
            fill_armed_next_bar: false,
            last_fill_bar: None,
            reinforcement_mode: Some(Tr909ReinforcementModeState::BreakReinforce),
        }
    }

    #[test]
    fn fixture_backed_render_policy_projection_holds() {
        let fixtures: Vec<RenderProjectionFixture> = serde_json::from_str(include_str!(
            "../../../riotbox-app/tests/fixtures/tr909_committed_render_projection.json"
        ))
        .expect("parse committed render projection fixture");

        let graph = sample_graph();
        for fixture in fixtures {
            let transport = transport_state(fixture.transport_position_beats);
            let scene_context = fixture.scene_context.as_deref().map(SceneId::from);
            let policy = derive_tr909_render_policy_with_scene_context(
                &Tr909LaneState {
                    pattern_ref: fixture.pattern_ref.clone(),
                    takeover_enabled: fixture.takeover_enabled,
                    takeover_profile: fixture.takeover_profile,
                    slam_enabled: false,
                    fill_armed_next_bar: false,
                    last_fill_bar: None,
                    reinforcement_mode: Some(fixture.reinforcement_mode),
                },
                &transport,
                Some(&graph),
                scene_context.as_ref(),
            );

            assert_eq!(
                policy.mode.label(),
                fixture.expected_mode,
                "{} mode",
                fixture.name
            );
            assert_eq!(
                policy.routing.label(),
                fixture.expected_routing,
                "{} routing",
                fixture.name
            );
            assert_eq!(
                policy
                    .pattern_adoption
                    .map(|value| value.label().to_string()),
                fixture.expected_pattern_adoption,
                "{} pattern adoption",
                fixture.name
            );
            assert_eq!(
                policy
                    .phrase_variation
                    .map(|value| value.label().to_string()),
                fixture.expected_phrase_variation,
                "{} phrase variation",
                fixture.name
            );
            assert_eq!(
                policy
                    .source_support_profile
                    .map(|value| value.label().to_string()),
                fixture.expected_source_support_profile,
                "{} support profile",
                fixture.name
            );
            assert_eq!(
                policy
                    .source_support_context
                    .map(|value| value.label().to_string()),
                fixture.expected_source_support_context,
                "{} support context",
                fixture.name
            );
            assert_eq!(
                policy
                    .takeover_profile
                    .map(|value| value.label().to_string()),
                fixture.expected_takeover_profile,
                "{} takeover profile",
                fixture.name
            );
        }
    }

    #[test]
    fn fill_next_owns_one_bar_then_returns_to_typed_reinforcement_mode() {
        let lane = Tr909LaneState {
            pattern_ref: Some("reinforce-scene-1".into()),
            takeover_enabled: false,
            takeover_profile: None,
            slam_enabled: true,
            fill_armed_next_bar: false,
            last_fill_bar: Some(4),
            reinforcement_mode: Some(Tr909ReinforcementModeState::BreakReinforce),
        };
        let fill_bar = derive_tr909_render_policy(&lane, &transport_state(12.0), None);
        let return_bar = derive_tr909_render_policy(&lane, &transport_state(16.0), None);

        assert_eq!(fill_bar.mode, Tr909RenderModePolicy::Fill);
        assert_eq!(return_bar.mode, Tr909RenderModePolicy::BreakReinforce);
        assert_eq!(return_bar.routing, Tr909RenderRoutingPolicy::DrumBusSupport);
    }

    #[test]
    fn source_support_profile_can_follow_projected_scene_context() {
        let graph = sample_graph();
        let transport = transport_state(0.0);
        let policy = derive_tr909_render_policy_with_scene_context(
            &Tr909LaneState {
                pattern_ref: Some("support-scene-02-break".into()),
                takeover_enabled: false,
                takeover_profile: None,
                slam_enabled: false,
                fill_armed_next_bar: false,
                last_fill_bar: None,
                reinforcement_mode: Some(Tr909ReinforcementModeState::SourceSupport),
            },
            &transport,
            Some(&graph),
            Some(&SceneId::from("scene-02-break")),
        );

        assert_eq!(transport.bar_index, 1);
        assert_eq!(
            policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::BreakLift)
        );
        assert_eq!(
            policy.source_support_context,
            Some(Tr909SourceSupportContextPolicy::SceneTarget)
        );
        assert_eq!(
            policy.pattern_adoption,
            Some(Tr909PatternAdoptionPolicy::SupportPulse)
        );
    }

    #[test]
    fn source_support_profile_falls_back_to_transport_for_unmapped_scene_context() {
        let graph = sample_graph();
        let transport = transport_state(4.0);
        let policy = derive_tr909_render_policy_with_scene_context(
            &Tr909LaneState {
                pattern_ref: Some("support-legacy-scene".into()),
                takeover_enabled: false,
                takeover_profile: None,
                slam_enabled: false,
                fill_armed_next_bar: false,
                last_fill_bar: None,
                reinforcement_mode: Some(Tr909ReinforcementModeState::SourceSupport),
            },
            &transport,
            Some(&graph),
            Some(&SceneId::from("scene-1")),
        );

        assert_eq!(
            policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::DropDrive)
        );
        assert_eq!(
            policy.source_support_context,
            Some(Tr909SourceSupportContextPolicy::TransportBar)
        );
    }

    #[test]
    fn feral_break_support_lifts_steady_source_support_profile() {
        let control_graph = steady_section_graph();
        let mut feral_graph = control_graph.clone();
        seed_feral_break_support(&mut feral_graph);
        let mut hook_only_graph = control_graph.clone();
        hook_only_graph.assets.push(Asset {
            asset_id: AssetId::from("asset-feral-hook-only"),
            asset_type: AssetType::HookFragment,
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.9,
            tags: vec!["feral".into()],
            source_refs: vec!["src-1".into()],
        });
        hook_only_graph.relationships.push(Relationship {
            relation_type: RelationshipType::SupportsBreakRebuild,
            from_id: "asset-feral-hook-only".into(),
            to_id: "section-steady".into(),
            weight: 0.85,
            notes: Some("feral hook supports rebuild".into()),
        });
        hook_only_graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        // Break-lift variation is intentionally checked on even phrase 2.
        let transport = transport_state(16.0);
        let tr909 = Tr909LaneState {
            pattern_ref: Some("support-feral-break".into()),
            takeover_enabled: false,
            takeover_profile: None,
            slam_enabled: false,
            fill_armed_next_bar: false,
            last_fill_bar: None,
            reinforcement_mode: Some(Tr909ReinforcementModeState::SourceSupport),
        };

        let control_policy = derive_tr909_render_policy_with_scene_context(
            &tr909,
            &transport,
            Some(&control_graph),
            None,
        );
        let feral_policy = derive_tr909_render_policy_with_scene_context(
            &tr909,
            &transport,
            Some(&feral_graph),
            None,
        );
        let hook_only_policy = derive_tr909_render_policy_with_scene_context(
            &tr909,
            &transport,
            Some(&hook_only_graph),
            None,
        );
        let control_reason =
            derive_tr909_source_support_reason(Some(&control_graph), &transport, None);
        let feral_reason = derive_tr909_source_support_reason(Some(&feral_graph), &transport, None);
        let hook_only_reason =
            derive_tr909_source_support_reason(Some(&hook_only_graph), &transport, None);

        assert_eq!(
            control_policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::SteadyPulse)
        );
        assert_eq!(control_reason, None);
        assert_eq!(
            feral_policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::BreakLift)
        );
        assert_eq!(
            feral_reason,
            Some(Tr909SourceSupportReasonPolicy::FeralBreakLift)
        );
        assert_eq!(
            hook_only_policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::BreakLift)
        );
        assert_eq!(
            hook_only_reason,
            Some(Tr909SourceSupportReasonPolicy::FeralBreakLift)
        );
        assert_eq!(
            feral_policy.source_support_context,
            Some(Tr909SourceSupportContextPolicy::TransportBar)
        );
        assert_eq!(
            feral_policy.phrase_variation,
            Some(Tr909PhraseVariationPolicy::PhraseLift)
        );
    }

    #[test]
    fn slam_selects_frozen_counter_rhythm_from_trusted_current_phrase() {
        let lane = break_reinforce_slam_lane();
        let eighth = derive_tr909_render_policy(
            &lane,
            &transport_state(0.0),
            Some(&counter_rhythm_graph(0.25, 0.25, 0.35)),
        );
        let late = derive_tr909_render_policy(
            &lane,
            &transport_state(0.0),
            Some(&counter_rhythm_graph(0.60, 0.55, 0.80)),
        );

        assert_eq!(
            eighth.counter_rhythm,
            Some(Tr909CounterRhythmPolicy::EighthAnswer)
        );
        assert_eq!(
            late.counter_rhythm,
            Some(Tr909CounterRhythmPolicy::LateSixteenthPickup)
        );
        assert_eq!(
            Tr909CounterRhythmPolicy::LateSixteenthPickup.label(),
            "late_sixteenth_pickup"
        );
    }

    #[test]
    fn counter_rhythm_fails_closed_without_every_activation_and_evidence_gate() {
        let transport = transport_state(0.0);
        let eligible = counter_rhythm_graph(0.45, 0.40, 0.80);
        let cases = [
            counter_rhythm_graph(0.249, 0.40, 0.80),
            counter_rhythm_graph(0.45, 0.249, 0.80),
            counter_rhythm_graph(0.45, 0.40, 0.349),
            counter_rhythm_graph(f32::NAN, 0.40, 0.80),
        ];

        for graph in &cases {
            assert_eq!(
                derive_tr909_render_policy(
                    &break_reinforce_slam_lane(),
                    &transport,
                    Some(graph)
                )
                .counter_rhythm,
                None
            );
        }

        let mut slam_off = break_reinforce_slam_lane();
        slam_off.slam_enabled = false;
        assert_eq!(
            derive_tr909_render_policy(&slam_off, &transport, Some(&eligible)).counter_rhythm,
            None
        );

        let mut stopped = transport;
        stopped.is_playing = false;
        assert_eq!(
            derive_tr909_render_policy(
                &break_reinforce_slam_lane(),
                &stopped,
                Some(&eligible)
            )
            .counter_rhythm,
            None
        );
    }

    #[test]
    fn counter_rhythm_selection_is_independent_of_source_filename() {
        let lane = break_reinforce_slam_lane();
        let transport = transport_state(0.0);
        let first = counter_rhythm_graph(0.50, 0.42, 0.75);
        let mut renamed = first.clone();
        renamed.source.path = "audio/completely-different-name.bin".into();

        assert_eq!(
            derive_tr909_render_policy(&lane, &transport, Some(&first)).counter_rhythm,
            derive_tr909_render_policy(&lane, &transport, Some(&renamed)).counter_rhythm
        );
    }

    #[test]
    fn exact_phrase_evidence_precedes_bar_range_fallback() {
        let mut graph = counter_rhythm_graph(0.50, 0.40, 0.60);
        let mut overlapping = graph.phrase_audio_features[0].clone();
        overlapping.phrase_index = 99;
        overlapping.offbeat_onset_density = 0.70;
        overlapping.confidence = 0.95;
        graph.phrase_audio_features.push(overlapping);

        let policy = derive_tr909_render_policy(
            &break_reinforce_slam_lane(),
            &transport_state(0.0),
            Some(&graph),
        );

        assert_eq!(
            policy.counter_rhythm,
            Some(Tr909CounterRhythmPolicy::EighthAnswer)
        );
    }
}
