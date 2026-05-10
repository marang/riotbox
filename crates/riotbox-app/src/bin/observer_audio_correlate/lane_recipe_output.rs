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
    mc202_phrase_grid: Option<Mc202PhraseGridEvidence>,
    mc202_phrase_grid_malformed: bool,
    mc202_source_phrase_slot: Option<Mc202SourcePhraseSlotEvidence>,
    mc202_source_phrase_slot_malformed: bool,
}

#[derive(Debug, PartialEq)]
pub(super) struct Mc202PhraseGridEvidence {
    hit_ratio: f64,
    starts_on_phrase_boundary: bool,
    candidate_onset_count: u64,
    grid_aligned_onset_count: u64,
    max_onset_offset_ms: f64,
    max_allowed_onset_offset_ms: f64,
    passed: bool,
}

#[derive(Debug, PartialEq)]
pub(super) struct Mc202SourcePhraseSlotEvidence {
    phrase_grid_available: bool,
    phrase_index: Option<u64>,
    starts_on_source_phrase_boundary: bool,
    passed: bool,
}

pub(super) fn collect_lane_recipe_cases(manifest: &Value) -> Vec<LaneRecipeCaseEvidence> {
    manifest["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|case| {
            let (mc202_phrase_grid, mc202_phrase_grid_malformed) = collect_mc202_phrase_grid(case);
            let (mc202_source_phrase_slot, mc202_source_phrase_slot_malformed) =
                collect_mc202_source_phrase_slot(case);
            LaneRecipeCaseEvidence {
                id: case["id"].as_str().unwrap_or("unknown").to_string(),
                result: case["result"].as_str().unwrap_or("unknown").to_string(),
                candidate_rms: case["metrics"]["candidate"]["rms"].as_f64(),
                signal_delta_rms: case["metrics"]["signal_delta"]["rms"].as_f64(),
                min_signal_delta_rms: case["thresholds"]["min_signal_delta_rms"].as_f64(),
                mc202_phrase_grid,
                mc202_phrase_grid_malformed,
                mc202_source_phrase_slot,
                mc202_source_phrase_slot_malformed,
            }
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

        if case.mc202_phrase_grid_malformed {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid=malformed",
                case.id
            ));
            continue;
        }

        let Some(phrase_grid) = &case.mc202_phrase_grid else {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid=missing",
                case.id
            ));
            continue;
        };
        if !phrase_grid.passed {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid=fail",
                case.id
            ));
        }
        if !phrase_grid.starts_on_phrase_boundary {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid.starts_on_phrase_boundary=false",
                case.id
            ));
        }
        if phrase_grid.candidate_onset_count == 0 {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid.candidate_onset_count=0",
                case.id
            ));
        }
        if phrase_grid.grid_aligned_onset_count == 0 {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid.grid_aligned_onset_count=0",
                case.id
            ));
        }
        if phrase_grid.hit_ratio < 0.95 {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid.hit_ratio={:.6} < 0.950000",
                case.id, phrase_grid.hit_ratio
            ));
        }
        if phrase_grid.max_onset_offset_ms > phrase_grid.max_allowed_onset_offset_ms {
            failures.push(format!(
                "lane_recipe_case={} mc202_phrase_grid.max_onset_offset_ms={:.6} > allowed {:.6}",
                case.id, phrase_grid.max_onset_offset_ms, phrase_grid.max_allowed_onset_offset_ms
            ));
        }

        if case.mc202_source_phrase_slot_malformed {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot=malformed",
                case.id
            ));
            continue;
        }

        let Some(source_phrase_slot) = &case.mc202_source_phrase_slot else {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot=missing",
                case.id
            ));
            continue;
        };
        if !source_phrase_slot.passed {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot=fail",
                case.id
            ));
        }
        if !source_phrase_slot.phrase_grid_available {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot.phrase_grid_available=false",
                case.id
            ));
        }
        if source_phrase_slot.phrase_index.is_none() {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot.phrase_index=missing",
                case.id
            ));
        }
        if !source_phrase_slot.starts_on_source_phrase_boundary {
            failures.push(format!(
                "lane_recipe_case={} mc202_source_phrase_slot.starts_on_source_phrase_boundary=false",
                case.id
            ));
        }
    }

    failures
}

fn collect_mc202_phrase_grid(case: &Value) -> (Option<Mc202PhraseGridEvidence>, bool) {
    let Some(metric) = case["metrics"].get("mc202_phrase_grid") else {
        return (None, false);
    };
    if metric.is_null() {
        return (None, false);
    }
    if !metric.is_object() {
        return (None, true);
    }

    let evidence = Mc202PhraseGridEvidence {
        hit_ratio: match metric["hit_ratio"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        starts_on_phrase_boundary: match metric["starts_on_phrase_boundary"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
        candidate_onset_count: match metric["candidate_onset_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        grid_aligned_onset_count: match metric["grid_aligned_onset_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_onset_offset_ms: match metric["max_onset_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_allowed_onset_offset_ms: match metric["max_allowed_onset_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        passed: match metric["passed"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn collect_mc202_source_phrase_slot(case: &Value) -> (Option<Mc202SourcePhraseSlotEvidence>, bool) {
    let Some(metric) = case["metrics"].get("mc202_source_phrase_slot") else {
        return (None, false);
    };
    if metric.is_null() {
        return (None, false);
    }
    if !metric.is_object() {
        return (None, true);
    }

    let evidence = Mc202SourcePhraseSlotEvidence {
        phrase_grid_available: match metric["phrase_grid_available"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
        phrase_index: metric["phrase_index"].as_u64(),
        starts_on_source_phrase_boundary: match metric["starts_on_source_phrase_boundary"].as_bool()
        {
            Some(value) => value,
            None => return (None, true),
        },
        passed: match metric["passed"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}
