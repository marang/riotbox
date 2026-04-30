use serde_json::Value;

const REQUIRED_RECIPE2_MC202_CASES: &[&str] = &[
    "mc202-follower-to-answer",
    "mc202-touch-low-to-high",
    "mc202-follower-to-pressure",
    "mc202-follower-to-instigator",
    "mc202-follower-to-mutated-drive",
    "mc202-neutral-to-lift-contour",
    "mc202-direct-to-hook-response",
];

#[derive(Debug, PartialEq)]
pub(super) struct LaneRecipeCaseEvidence {
    id: String,
    result: String,
    candidate_rms: Option<f64>,
    signal_delta_rms: Option<f64>,
    min_signal_delta_rms: Option<f64>,
}

pub(super) fn collect_lane_recipe_cases(manifest: &Value) -> Vec<LaneRecipeCaseEvidence> {
    manifest["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|case| LaneRecipeCaseEvidence {
            id: case["id"].as_str().unwrap_or("unknown").to_string(),
            result: case["result"].as_str().unwrap_or("unknown").to_string(),
            candidate_rms: case["metrics"]["candidate"]["rms"].as_f64(),
            signal_delta_rms: case["metrics"]["signal_delta"]["rms"].as_f64(),
            min_signal_delta_rms: case["thresholds"]["min_signal_delta_rms"].as_f64(),
        })
        .collect()
}

pub(super) fn lane_recipe_metric_failures(
    cases: &[LaneRecipeCaseEvidence],
    metric_floor: f64,
) -> Vec<String> {
    let mut failures = Vec::new();

    for required_id in REQUIRED_RECIPE2_MC202_CASES {
        let Some(case) = cases.iter().find(|case| case.id == *required_id) else {
            failures.push(format!("lane_recipe_case={required_id}=missing"));
            continue;
        };

        if case.result != "pass" {
            failures.push(format!(
                "lane_recipe_case={} result={}",
                case.id, case.result
            ));
        }

        match case.candidate_rms {
            Some(value) if value > metric_floor => {}
            Some(value) => failures.push(format!(
                "lane_recipe_case={} candidate_rms={value:.6} <= floor {metric_floor:.6}",
                case.id
            )),
            None => failures.push(format!(
                "lane_recipe_case={} candidate_rms=missing",
                case.id
            )),
        }

        match (case.signal_delta_rms, case.min_signal_delta_rms) {
            (Some(delta), Some(minimum)) if delta >= minimum && delta > metric_floor => {}
            (Some(delta), Some(minimum)) => failures.push(format!(
                "lane_recipe_case={} signal_delta_rms={delta:.6} < min {minimum:.6}",
                case.id
            )),
            (None, _) => failures.push(format!(
                "lane_recipe_case={} signal_delta_rms=missing",
                case.id
            )),
            (_, None) => failures.push(format!(
                "lane_recipe_case={} min_signal_delta_rms=missing",
                case.id
            )),
        }
    }

    failures
}
