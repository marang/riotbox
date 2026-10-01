use super::report_model::SourceTimingEvidence;
use super::source_timing_anchor_evidence::collect_optional_source_timing_anchor_evidence;
use super::source_timing_groove_evidence::collect_optional_source_timing_groove_evidence;
use super::value_fields::non_empty_string;
use super::value_fields::string_list;
use serde_json::Value;

pub(super) fn collect_source_timing(manifest: &Value) -> (Option<SourceTimingEvidence>, bool) {
    let Some(source_timing) = manifest.get("source_timing") else {
        return (None, false);
    };
    if !source_timing.is_object() {
        return (None, true);
    }

    let evidence = SourceTimingEvidence {
        source_id: match source_timing_string(source_timing, "source_id") {
            Some(value) => value,
            None => return (None, true),
        },
        policy_profile: match source_timing_string(source_timing, "policy_profile") {
            Some(value) => value,
            None => return (None, true),
        },
        actionability: match optional_source_timing_string(source_timing, "actionability") {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        grid_use: match optional_source_timing_string(source_timing, "grid_use") {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        readiness: match source_timing_string(source_timing, "readiness") {
            Some(value) => value,
            None => return (None, true),
        },
        requires_manual_confirm: match source_timing["requires_manual_confirm"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
        primary_bpm: match source_timing.get("primary_bpm") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_f64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        bpm_agrees_with_grid: match source_timing.get("bpm_agrees_with_grid") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_bool() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        beat_status: match source_timing_string(source_timing, "beat_status") {
            Some(value) => value,
            None => return (None, true),
        },
        downbeat_status: match source_timing_string(source_timing, "downbeat_status") {
            Some(value) => value,
            None => return (None, true),
        },
        primary_downbeat_offset_beats: match source_timing.get("primary_downbeat_offset_beats") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_u64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        primary_downbeat_score: match optional_manifest_source_timing_f64(
            source_timing,
            "primary_downbeat_score",
        ) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        primary_downbeat_margin: match optional_manifest_source_timing_f64(
            source_timing,
            "primary_downbeat_margin",
        ) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        alternate_downbeat_phase_count: match optional_manifest_source_timing_u64(
            source_timing,
            "alternate_downbeat_phase_count",
        ) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        confidence_result: match source_timing_string(source_timing, "confidence_result") {
            Some(value) => value,
            None => return (None, true),
        },
        drift_status: match source_timing_string(source_timing, "drift_status") {
            Some(value) => value,
            None => return (None, true),
        },
        phrase_status: match source_timing_string(source_timing, "phrase_status") {
            Some(value) => value,
            None => return (None, true),
        },
        primary_phrase_count: match source_timing["primary_phrase_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        primary_phrase_bar_count: match source_timing["primary_phrase_bar_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        alternate_evidence_count: match source_timing["alternate_evidence_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        anchor_evidence: match collect_optional_source_timing_anchor_evidence(source_timing) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        groove_evidence: match collect_optional_source_timing_groove_evidence(source_timing) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        warning_codes: match string_list(source_timing, "warning_codes") {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn source_timing_string(source_timing: &Value, field: &str) -> Option<String> {
    non_empty_string(source_timing, field)
}

fn optional_source_timing_string(source_timing: &Value, field: &str) -> Result<Option<String>, ()> {
    match source_timing.get(field) {
        Some(value) if value.is_null() => Ok(None),
        Some(value) => value
            .as_str()
            .filter(|value| !value.is_empty())
            .map(|value| Some(value.to_string()))
            .ok_or(()),
        None => Ok(None),
    }
}

fn optional_manifest_source_timing_f64(
    source_timing: &Value,
    field: &str,
) -> Result<Option<f64>, ()> {
    match source_timing.get(field) {
        Some(value) if value.is_null() => Ok(None),
        Some(value) => value.as_f64().map(Some).ok_or(()),
        None => Ok(None),
    }
}

fn optional_manifest_source_timing_u64(
    source_timing: &Value,
    field: &str,
) -> Result<Option<u64>, ()> {
    match source_timing.get(field) {
        Some(value) if value.is_null() => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or(()),
        None => Ok(None),
    }
}
