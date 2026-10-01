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
    if evidence.typed_anchor_count().ok_or(())? > evidence.primary_anchor_count {
        return Err(());
    }

    Ok(Some(evidence))
}

impl SourceTimingAnchorEvidence {
    fn typed_anchor_count(&self) -> Option<u64> {
        self.primary_kick_anchor_count
            .checked_add(self.primary_backbeat_anchor_count)
            .and_then(|count| count.checked_add(self.primary_transient_anchor_count))
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

#[cfg(test)]
mod tests {
    use super::collect_optional_source_timing_anchor_evidence;
    use super::source_timing_anchor_evidence_json;
    use serde_json::json;

    #[test]
    fn overflowing_kick_and_backbeat_anchor_total_is_rejected() {
        let timing = json!({
            "anchor_evidence": {
                "primary_anchor_count": u64::MAX,
                "primary_kick_anchor_count": u64::MAX,
                "primary_backbeat_anchor_count": 1,
                "primary_transient_anchor_count": 0,
            }
        });

        assert_eq!(
            collect_optional_source_timing_anchor_evidence(&timing),
            Err(())
        );
    }

    #[test]
    fn overflowing_transient_anchor_total_is_rejected() {
        let timing = json!({
            "anchor_evidence": {
                "primary_anchor_count": u64::MAX,
                "primary_kick_anchor_count": u64::MAX - 1,
                "primary_backbeat_anchor_count": 1,
                "primary_transient_anchor_count": 1,
            }
        });

        assert_eq!(
            collect_optional_source_timing_anchor_evidence(&timing),
            Err(())
        );
    }

    #[test]
    fn representable_boundary_totals_preserve_anchor_evidence() {
        for (total, kick, backbeat, transient) in [
            (0, 0, 0, 0),
            (8, 2, 4, 2),
            (9, 2, 4, 2),
            (u64::MAX, u64::MAX, 0, 0),
            (u64::MAX, 0, u64::MAX, 0),
            (u64::MAX, 0, 0, u64::MAX),
            (u64::MAX, u64::MAX - 2, 1, 1),
        ] {
            let timing = json!({
                "anchor_evidence": {
                    "primary_anchor_count": total,
                    "primary_kick_anchor_count": kick,
                    "primary_backbeat_anchor_count": backbeat,
                    "primary_transient_anchor_count": transient,
                }
            });
            let evidence = collect_optional_source_timing_anchor_evidence(&timing)
                .expect("representable total")
                .expect("present evidence");

            assert_eq!(
                source_timing_anchor_evidence_json(&evidence),
                timing["anchor_evidence"]
            );
        }
    }

    #[test]
    fn representable_typed_total_exceeding_declared_count_is_rejected() {
        for (total, kick, backbeat, transient) in [
            (0, 1, 0, 0),
            (7, 2, 4, 2),
            (u64::MAX - 1, u64::MAX - 2, 1, 1),
        ] {
            let timing = json!({
                "anchor_evidence": {
                    "primary_anchor_count": total,
                    "primary_kick_anchor_count": kick,
                    "primary_backbeat_anchor_count": backbeat,
                    "primary_transient_anchor_count": transient,
                }
            });

            assert_eq!(
                collect_optional_source_timing_anchor_evidence(&timing),
                Err(())
            );
        }
    }

    #[test]
    fn missing_or_null_anchor_evidence_remains_optional() {
        for timing in [json!({}), json!({"anchor_evidence": null})] {
            assert_eq!(
                collect_optional_source_timing_anchor_evidence(&timing),
                Ok(None)
            );
        }
    }

    #[test]
    fn non_u64_anchor_fields_are_rejected_without_conversion() {
        for field in [
            "primary_anchor_count",
            "primary_kick_anchor_count",
            "primary_backbeat_anchor_count",
            "primary_transient_anchor_count",
        ] {
            for invalid in [json!(-1), json!(1.5), json!("1"), json!(true), json!(null)] {
                let mut timing = json!({
                    "anchor_evidence": {
                        "primary_anchor_count": 8,
                        "primary_kick_anchor_count": 2,
                        "primary_backbeat_anchor_count": 4,
                        "primary_transient_anchor_count": 2,
                    }
                });
                timing["anchor_evidence"][field] = invalid;
                assert_eq!(
                    collect_optional_source_timing_anchor_evidence(&timing),
                    Err(())
                );
            }
        }
    }
}
