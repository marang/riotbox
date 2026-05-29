#[cfg(test)]
mod manifest_mix_assertions {
    pub(super) fn assert_all_lane_mix_movement(manifest: &serde_json::Value) {
        let mix_movement = &manifest["metrics"]["all_lane_mix_movement"];
        assert_eq!(mix_movement["applied"], true);
        assert_eq!(mix_movement["reason"], "all_lane_mix_movement_proof");
        assert!(
            mix_movement["source_first_to_support_rms_delta"]
                .as_f64()
                .expect("source-first/support rms delta")
                >= mix_movement["min_required_rms_delta"]
                    .as_f64()
                    .expect("min mix rms delta")
        );
        assert!(
            mix_movement["source_first_to_support_correlation"]
                .as_f64()
                .expect("source-first/support correlation")
                <= mix_movement["max_allowed_correlation"]
                    .as_f64()
                    .expect("max mix correlation")
        );

        for key in [
            "tr909_contribution_ratio",
            "mc202_contribution_ratio",
            "w30_contribution_ratio",
        ] {
            assert!(
                mix_movement[key]
                    .as_f64()
                    .unwrap_or_else(|| panic!("{key}"))
                    >= mix_movement["min_required_lane_contribution_ratio"]
                        .as_f64()
                        .expect("min lane contribution")
            );
        }
        assert!(
            mix_movement["generated_to_w30_contribution_ratio"]
                .as_f64()
                .expect("generated/w30 contribution")
                >= mix_movement["min_required_generated_to_w30_ratio"]
                    .as_f64()
                    .expect("min generated/w30 contribution")
        );
    }
}
