use super::report_model::SourceGridOutputDriftEvidence;
use super::report_model::W30SourceLoopClosureEvidence;
use serde_json::Value;

pub(super) fn collect_source_grid_output_drift(
    manifest: &Value,
) -> (Option<SourceGridOutputDriftEvidence>, bool) {
    collect_source_grid_alignment(manifest, "source_grid_output_drift")
}

pub(super) fn collect_source_grid_alignment(
    manifest: &Value,
    metric_key: &str,
) -> (Option<SourceGridOutputDriftEvidence>, bool) {
    let Some(metrics) = manifest.get("metrics").and_then(Value::as_object) else {
        return (None, false);
    };
    let Some(metric) = metrics.get(metric_key) else {
        return (None, false);
    };

    let evidence = SourceGridOutputDriftEvidence {
        hit_ratio: match metric["hit_ratio"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_peak_offset_ms: match metric["max_peak_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_allowed_peak_offset_ms: match metric["max_allowed_peak_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

pub(super) fn collect_metric_string(
    manifest: &Value,
    metric_key: &str,
    field: &str,
) -> Option<String> {
    manifest
        .get("metrics")?
        .get(metric_key)?
        .get(field)?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn collect_metric_bool(manifest: &Value, metric_key: &str, field: &str) -> Option<bool> {
    manifest
        .get("metrics")?
        .get(metric_key)?
        .get(field)?
        .as_bool()
}

pub(super) fn collect_metric_f64(manifest: &Value, metric_key: &str, field: &str) -> Option<f64> {
    manifest
        .get("metrics")?
        .get(metric_key)?
        .get(field)?
        .as_f64()
}

pub(super) fn collect_w30_source_loop_closure(
    manifest: &Value,
) -> (Option<W30SourceLoopClosureEvidence>, bool) {
    let Some(metrics) = manifest.get("metrics").and_then(Value::as_object) else {
        return (None, false);
    };
    let Some(metric) = metrics.get("w30_source_loop_closure") else {
        return (None, false);
    };

    let evidence = W30SourceLoopClosureEvidence {
        passed: match metric["passed"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
        preview_rms: match metric["preview_rms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        edge_delta_abs: match metric["edge_delta_abs"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_allowed_edge_delta_abs: match metric["max_allowed_edge_delta_abs"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        edge_abs_max: match metric["edge_abs_max"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_allowed_edge_abs: match metric["max_allowed_edge_abs"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        source_contains_selection: match metric["source_contains_selection"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}
