#[cfg(test)]
mod manifest_mc202_assertions {
    use super::MIN_SIGNAL_RMS;

    pub(super) fn assert_mc202_manifest(manifest: &serde_json::Value) {
        let mc202_bass_pressure = &manifest["metrics"]["mc202_bass_pressure"];
        assert_eq!(mc202_bass_pressure["pattern_origin"], "primitive_renderer");
        assert_eq!(mc202_bass_pressure["applied"], true);
        assert_eq!(
            mc202_bass_pressure["pressure_role"],
            "bass_pressure_with_source_contour"
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
                    "bass_pressure" | "answer_lift" | "hook_restraint_hold"
                )
            }));
        assert_eq!(mc202_bass_pressure["phrase_variation_applied"], true);
        assert_eq!(
            mc202_bass_pressure["reason"],
            "mc202_source_grid_proof_renderer"
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
            "source_derived_contour"
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
