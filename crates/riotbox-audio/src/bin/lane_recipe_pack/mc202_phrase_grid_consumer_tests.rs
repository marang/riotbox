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
}
