#[cfg(test)]
mod manifest_mc202_assertions {
    use super::{
        MC202_PATTERN_ORIGIN_PRIMITIVE_RENDERER, MC202_PATTERN_ORIGIN_SOURCE_DERIVED_CONTOUR,
        MC202_PRESSURE_ROLE_WITH_SOURCE_CONTOUR, MC202_REASON_SOURCE_GRID_PROOF_RENDERER,
        MC202_SOURCE_EXPRESSION_ROLE_ANSWER_LIFT, MC202_SOURCE_EXPRESSION_ROLE_BASS_PRESSURE,
        MC202_SOURCE_EXPRESSION_ROLE_HOOK_RESTRAINT_HOLD, MIN_SIGNAL_RMS,
    };

    pub(super) fn assert_mc202_manifest(manifest: &serde_json::Value) {
        let mc202_bass_pressure = &manifest["metrics"]["mc202_bass_pressure"];
        assert_eq!(
            mc202_bass_pressure["pattern_origin"],
            MC202_PATTERN_ORIGIN_PRIMITIVE_RENDERER
        );
        assert_eq!(mc202_bass_pressure["applied"], true);
        assert_eq!(
            mc202_bass_pressure["pressure_role"],
            MC202_PRESSURE_ROLE_WITH_SOURCE_CONTOUR
        );
        assert_eq!(
            mc202_bass_pressure["source_expression_render_plan_applied"],
            true
        );
        assert!(mc202_bass_pressure["source_expression_role"]
            .as_str()
            .is_some_and(|role| {
                matches!(
                    role,
                    MC202_SOURCE_EXPRESSION_ROLE_BASS_PRESSURE
                        | MC202_SOURCE_EXPRESSION_ROLE_ANSWER_LIFT
                        | MC202_SOURCE_EXPRESSION_ROLE_HOOK_RESTRAINT_HOLD
                )
            }));
        assert_eq!(mc202_bass_pressure["phrase_variation_applied"], true);
        assert_eq!(
            mc202_bass_pressure["reason"],
            MC202_REASON_SOURCE_GRID_PROOF_RENDERER
        );
        assert!(
            mc202_bass_pressure["pressure_reinforcement_gain"]
                .as_f64()
                .expect("mc202 pressure reinforcement gain")
                > 0.0
        );
        assert!(
            mc202_bass_pressure["low_to_mid_energy_ratio"]
                .as_f64()
                .expect("mc202 low/mid energy ratio")
                >= mc202_bass_pressure["min_low_to_mid_energy_ratio"]
                    .as_f64()
                    .expect("mc202 min low/mid energy ratio")
        );

        let mc202_source_contour = &manifest["metrics"]["mc202_source_contour"];
        assert_eq!(
            mc202_source_contour["pattern_origin"],
            MC202_PATTERN_ORIGIN_SOURCE_DERIVED_CONTOUR
        );
        assert_eq!(mc202_source_contour["applied"], true);
        assert!(mc202_source_contour["reason"]
            .as_str()
            .is_some_and(|reason| reason.starts_with("source_")));
        assert!(
            mc202_source_contour["source_contour_delta_rms"]
                .as_f64()
                .expect("mc202 source contour delta")
                >= mc202_source_contour["min_required_delta_rms"]
                    .as_f64()
                    .expect("mc202 source contour min delta")
        );
        assert!(
            manifest["metrics"]["mc202_bass_pressure_stem"]["signal"]["rms"]
                .as_f64()
                .expect("mc202 stem rms")
                > f64::from(MIN_SIGNAL_RMS)
        );
    }
}
