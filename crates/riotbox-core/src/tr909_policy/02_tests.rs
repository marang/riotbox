#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{
        ids::{AssetId, SceneId, SectionId, SourceId},
        session::{Tr909LaneState, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
        source_graph::{
            Asset, AssetType, Candidate, CandidateType, DecodeProfile, EnergyClass,
            GraphProvenance, QualityClass, Relationship, RelationshipType, Section,
            SectionLabelHint, SourceDescriptor, SourceGraph,
        },
        transport::TransportClockState,
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
        let beat_index = position_beats.floor() as u64;
        let bar_index = beat_index / 4;
        let phrase_index = bar_index / 8;
        TransportClockState {
            is_playing: true,
            position_beats,
            beat_index,
            bar_index,
            phrase_index,
            current_scene: Some(SceneId::from("scene-1")),
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
    fn source_support_profile_can_follow_projected_scene_context() {
        let graph = sample_graph();
        let transport = transport_state(4.0);
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
        let transport = transport_state(4.0);
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
}
