use super::report_model::SourceTimingAnchorEvidence;
use super::value_fields::u64_field;
use serde_json::Value;

pub(super) fn collect_optional_source_timing_anchor_evidence(
    source_timing: &Value,
) -> Result<Option<SourceTimingAnchorEvidence>, ()> {
    let Some(value) = source_timing.get("anchor_evidence") else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(anchor_evidence) = value.as_object() else {
        return Err(());
    };

    let evidence = SourceTimingAnchorEvidence {
        primary_anchor_count: u64_field(anchor_evidence, "primary_anchor_count")?,
        primary_kick_anchor_count: u64_field(anchor_evidence, "primary_kick_anchor_count")?,
        primary_backbeat_anchor_count: u64_field(anchor_evidence, "primary_backbeat_anchor_count")?,
        primary_transient_anchor_count: u64_field(
            anchor_evidence,
            "primary_transient_anchor_count",
        )?,
    };
    if evidence.typed_anchor_count() > evidence.primary_anchor_count {
        return Err(());
    }

    Ok(Some(evidence))
}

impl SourceTimingAnchorEvidence {
    fn typed_anchor_count(&self) -> u64 {
        self.primary_kick_anchor_count
            + self.primary_backbeat_anchor_count
            + self.primary_transient_anchor_count
    }
}

pub(super) fn source_timing_anchor_evidence_json(
    evidence: &SourceTimingAnchorEvidence,
) -> serde_json::Value {
    serde_json::json!({
        "primary_anchor_count": evidence.primary_anchor_count,
        "primary_kick_anchor_count": evidence.primary_kick_anchor_count,
        "primary_backbeat_anchor_count": evidence.primary_backbeat_anchor_count,
        "primary_transient_anchor_count": evidence.primary_transient_anchor_count,
    })
}
