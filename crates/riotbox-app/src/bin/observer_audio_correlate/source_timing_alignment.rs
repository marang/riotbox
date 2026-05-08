#[derive(Debug, PartialEq)]
struct SourceTimingAlignmentEvidence {
    status: String,
    bpm_delta: Option<f64>,
    bpm_tolerance: f64,
    warning_overlap: Vec<String>,
    issues: Vec<String>,
}

fn collect_source_timing_alignment(
    observer_timing: Option<&ObserverSourceTimingReadiness>,
    manifest_timing: Option<&SourceTimingEvidence>,
    observer_malformed: bool,
    manifest_malformed: bool,
) -> Option<SourceTimingAlignmentEvidence> {
    if observer_malformed || manifest_malformed {
        return None;
    }
    let (Some(observer), Some(manifest)) = (observer_timing, manifest_timing) else {
        return None;
    };

    let bpm_delta = match (observer.bpm_estimate, manifest.primary_bpm) {
        (Some(observer_bpm), Some(manifest_bpm)) => Some((observer_bpm - manifest_bpm).abs()),
        _ => None,
    };
    let observer_warnings = normalize_warning_codes(&observer.warning_codes);
    let manifest_warnings = normalize_warning_codes(&manifest.warning_codes);
    let warning_overlap = observer_warnings
        .iter()
        .filter(|warning| manifest_warnings.contains(*warning))
        .cloned()
        .collect::<Vec<_>>();

    let mut issues = Vec::new();
    if let Some(delta) = bpm_delta
        && delta > SOURCE_TIMING_BPM_ALIGNMENT_TOLERANCE
    {
        issues.push(format!(
            "source_timing_alignment.bpm_delta={delta:.6} > tolerance {SOURCE_TIMING_BPM_ALIGNMENT_TOLERANCE:.6}"
        ));
    }
    if !observer_warnings.is_empty() && !manifest_warnings.is_empty() && warning_overlap.is_empty()
    {
        issues.push("source_timing_alignment.warning_codes=no_overlap".to_string());
    }

    let status = if !issues.is_empty() {
        "mismatch"
    } else if bpm_delta.is_some() || !warning_overlap.is_empty() {
        "aligned"
    } else {
        "partial"
    };

    Some(SourceTimingAlignmentEvidence {
        status: status.to_string(),
        bpm_delta,
        bpm_tolerance: SOURCE_TIMING_BPM_ALIGNMENT_TOLERANCE,
        warning_overlap,
        issues,
    })
}

fn normalize_warning_codes(codes: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for code in codes {
        let value = normalize_warning_code(code);
        if !value.is_empty() && !normalized.contains(&value) {
            normalized.push(value);
        }
    }
    normalized
}

fn normalize_warning_code(code: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = true;
    for character in code.chars() {
        if character == '_' || character == '-' || character == ' ' {
            if !previous_was_separator && !normalized.is_empty() {
                normalized.push('_');
            }
            previous_was_separator = true;
        } else if character.is_ascii_uppercase() {
            if !previous_was_separator && !normalized.is_empty() {
                normalized.push('_');
            }
            normalized.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        }
    }
    normalized
        .trim_matches('_')
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
