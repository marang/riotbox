#[derive(Clone, Debug, PartialEq)]
struct SourceTimingGrooveEvidence {
    primary_groove_residual_count: u64,
    primary_max_abs_offset_ms: f64,
    primary_groove_preview: Vec<SourceTimingGrooveResidualEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
struct SourceTimingGrooveResidualEvidence {
    subdivision: String,
    offset_ms: f64,
    confidence: f64,
}

fn collect_optional_source_timing_groove_evidence(
    source_timing: &Value,
) -> Result<Option<SourceTimingGrooveEvidence>, ()> {
    let Some(value) = source_timing.get("groove_evidence") else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(groove_evidence) = value.as_object() else {
        return Err(());
    };

    let evidence = SourceTimingGrooveEvidence {
        primary_groove_residual_count: u64_field(
            groove_evidence,
            "primary_groove_residual_count",
        )?,
        primary_max_abs_offset_ms: non_negative_f64_field(
            groove_evidence,
            "primary_max_abs_offset_ms",
        )?,
        primary_groove_preview: collect_source_timing_groove_preview(groove_evidence)?,
    };
    if evidence.primary_groove_preview.len()
        > usize::min(evidence.primary_groove_residual_count as usize, 4)
    {
        return Err(());
    }

    Ok(Some(evidence))
}

fn collect_source_timing_groove_preview(
    groove_evidence: &serde_json::Map<String, Value>,
) -> Result<Vec<SourceTimingGrooveResidualEvidence>, ()> {
    let Some(preview) = groove_evidence
        .get("primary_groove_preview")
        .and_then(Value::as_array)
    else {
        return Err(());
    };

    preview
        .iter()
        .map(collect_source_timing_groove_residual)
        .collect()
}

fn collect_source_timing_groove_residual(
    value: &Value,
) -> Result<SourceTimingGrooveResidualEvidence, ()> {
    let Some(residual) = value.as_object() else {
        return Err(());
    };
    let Some(subdivision) = residual
        .get("subdivision")
        .and_then(Value::as_str)
        .filter(|value| {
            matches!(
                *value,
                "eighth" | "triplet" | "sixteenth" | "thirty_second"
            )
        })
    else {
        return Err(());
    };
    let confidence = non_negative_f64_field(residual, "confidence")?;
    if confidence > 1.0 {
        return Err(());
    }

    Ok(SourceTimingGrooveResidualEvidence {
        subdivision: subdivision.to_string(),
        offset_ms: f64_field(residual, "offset_ms")?,
        confidence,
    })
}

fn f64_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<f64, ()> {
    object.get(field).and_then(Value::as_f64).ok_or(())
}

fn non_negative_f64_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<f64, ()> {
    let value = f64_field(object, field)?;
    if value < 0.0 {
        return Err(());
    }
    Ok(value)
}

fn source_timing_groove_evidence_json(
    evidence: &SourceTimingGrooveEvidence,
) -> serde_json::Value {
    serde_json::json!({
        "primary_groove_residual_count": evidence.primary_groove_residual_count,
        "primary_max_abs_offset_ms": evidence.primary_max_abs_offset_ms,
        "primary_groove_preview": evidence.primary_groove_preview.iter().map(source_timing_groove_residual_json).collect::<Vec<_>>(),
    })
}

fn source_timing_groove_residual_json(
    residual: &SourceTimingGrooveResidualEvidence,
) -> serde_json::Value {
    serde_json::json!({
        "subdivision": &residual.subdivision,
        "offset_ms": residual.offset_ms,
        "confidence": residual.confidence,
    })
}
