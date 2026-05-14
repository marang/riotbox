#[cfg(test)]
mod mc202_phrase_grid_consumer_tests {
    use super::*;

    #[test]
    fn mc202_lane_recipe_cases_consume_phrase_grid_metrics() {
        let mc202_cases = pack_cases()
            .into_iter()
            .filter(|case| matches!(case.render_pair, RenderPair::Mc202 { .. }))
            .collect::<Vec<_>>();

        assert!(
            mc202_cases.len() >= 7,
            "expected current MC-202 recipe coverage"
        );
        for case in mc202_cases {
            let (_, candidate) = render_pair(&case.render_pair, 88_200);
            let metrics = mc202_phrase_grid_metrics(&case.render_pair, &candidate)
                .unwrap_or_else(|| panic!("{} missing MC-202 phrase-grid metrics", case.id));

            assert!(
                metrics.passed,
                "{} should pass MC-202 phrase-grid timing: {:?}",
                case.id, metrics
            );
            assert!(
                metrics.starts_on_phrase_boundary,
                "{} should start on a phrase boundary",
                case.id
            );
            assert!(
                metrics.candidate_onset_count > 0,
                "{} should render audible MC-202 onsets",
                case.id
            );
            assert!(
                metrics.hit_ratio >= MC202_PHRASE_GRID_MIN_HIT_RATIO,
                "{} hit ratio {} should stay on the sixteenth phrase grid",
                case.id,
                metrics.hit_ratio
            );
            assert!(
                metrics.max_onset_offset_ms <= MC202_PHRASE_GRID_MAX_ONSET_OFFSET_MS,
                "{} max onset offset {} ms should stay inside phrase-grid tolerance",
                case.id,
                metrics.max_onset_offset_ms
            );
        }
    }

    #[test]
    fn mc202_lane_recipe_cases_consume_source_phrase_slots() {
        let source_timing = lane_recipe_source_timing_model();
        let mc202_cases = pack_cases()
            .into_iter()
            .filter(|case| matches!(case.render_pair, RenderPair::Mc202 { .. }))
            .collect::<Vec<_>>();

        for case in mc202_cases {
            let metrics = mc202_source_phrase_slot_metrics(&case.render_pair, &source_timing)
                .unwrap_or_else(|| panic!("{} missing MC-202 source phrase-slot metrics", case.id));

            assert!(
                metrics.passed,
                "{} should land on a selected source phrase slot: {:?}",
                case.id, metrics
            );
            assert_eq!(metrics.source_hypothesis_id.as_deref(), Some("probe-bpm-primary"));
            assert_eq!(metrics.phrase_index, Some(3));
            assert_eq!(metrics.phrase_start_bar, Some(9));
            assert!(metrics.starts_on_source_phrase_boundary);
        }
    }

    #[test]
    fn lane_recipe_source_timing_uses_probe_generated_phrase_grid() {
        let source_timing = lane_recipe_source_timing_model();
        let primary = source_timing.primary_hypothesis().expect("primary hypothesis");

        assert_eq!(source_timing.primary_hypothesis_id.as_deref(), Some("probe-bpm-primary"));
        assert!(
            primary
                .provenance
                .contains(&"source-timing-probe.phrase-grid.v0".to_string())
        );
        assert!(
            primary
                .provenance
                .contains(&"lane-recipe-pack.generated-source-phrase-grid.v1".to_string())
        );
        assert_eq!(source_timing.phrase_grid.len(), 3);
        assert_eq!(source_timing.phrase_grid[2].start_bar, 9);
    }

    #[test]
    fn mc202_phrase_grid_gate_rejects_non_phrase_boundary_candidate() {
        let mut candidate = mc202_state(Mc202RenderMode::Answer, Mc202PhraseShape::AnswerHook, 0.78);
        candidate.position_beats = 4.0;
        let pair = RenderPair::Mc202 {
            baseline: mc202_state(Mc202RenderMode::Follower, Mc202PhraseShape::FollowerDrive, 0.62),
            candidate,
        };
        let (_, candidate_samples) = render_pair(&pair, 88_200);

        let metrics = mc202_phrase_grid_metrics(&pair, &candidate_samples).expect("MC-202 metrics");

        assert!(
            !metrics.starts_on_phrase_boundary,
            "bar-aligned but phrase-offset MC-202 candidate must not claim phrase-boundary alignment"
        );
        assert!(
            !metrics.passed,
            "phrase-offset MC-202 candidate must fail the phrase-grid timing gate"
        );
    }

    #[test]
    fn mc202_source_phrase_slot_gate_rejects_non_source_phrase_boundary_candidate() {
        let source_timing = lane_recipe_source_timing_model();
        let mut candidate = mc202_state(Mc202RenderMode::Answer, Mc202PhraseShape::AnswerHook, 0.78);
        candidate.position_beats = 28.0;
        let pair = RenderPair::Mc202 {
            baseline: mc202_state(Mc202RenderMode::Follower, Mc202PhraseShape::FollowerDrive, 0.62),
            candidate,
        };

        let metrics =
            mc202_source_phrase_slot_metrics(&pair, &source_timing).expect("source phrase metrics");

        assert_eq!(metrics.phrase_index, Some(2));
        assert!(
            !metrics.starts_on_source_phrase_boundary,
            "candidate inside phrase but not at the source phrase boundary must be visible"
        );
        assert!(
            !metrics.passed,
            "source phrase-offset MC-202 candidate must fail the source phrase-slot timing gate"
        );
    }
}
