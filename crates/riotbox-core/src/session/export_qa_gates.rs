use serde::{Deserialize, Serialize};

use super::export_types::ExportArtifactRole;

pub const PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID: &str = "product_export_reproducibility_smoke";
pub const STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID: &str = "stem_package_artifact_set_evidence";
pub const STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID: &str = "stem_package_per_stem_hash_stability";
pub const STEM_PACKAGE_NON_SILENCE_QA_GATE_ID: &str = "stem_package_per_stem_non_silence";
pub const STEM_PACKAGE_LINEAGE_QA_GATE_ID: &str = "stem_package_per_stem_lineage";
pub const STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID: &str =
    "stem_package_per_stem_fallback_comparison";
pub const DAW_SESSION_JSON_PACKAGE_QA_GATE_ID: &str = "daw_session_json_package_integrity";
pub const DAW_SESSION_HOST_IMPORT_QA_GATE_ID: &str = "daw_session_host_import_proof";
pub const DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID: &str = "daw_session_audible_output_proof";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReceiptQaGateResult {
    pub gate_id: String,
    pub status: ExportReceiptQaGateStatus,
    #[serde(default)]
    pub artifact_roles: Vec<ExportArtifactRole>,
    #[serde(default)]
    pub summary: Option<String>,
}

impl ExportReceiptQaGateResult {
    #[must_use]
    pub fn product_export_reproducibility() -> Self {
        Self {
            gate_id: PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID.into(),
            status: ExportReceiptQaGateStatus::Passed,
            artifact_roles: vec![ExportArtifactRole::FullGridMix],
            summary: Some("product export proof and artifact hash accepted".into()),
        }
    }

    #[must_use]
    pub fn stem_package_artifact_set_evidence(
        report: &crate::export_qa::StemPackageArtifactSetQaReport,
    ) -> Self {
        let status = if report.status == crate::export_qa::StemPackageArtifactSetQaStatus::Failed {
            ExportReceiptQaGateStatus::Failed
        } else if report.deferred_checks.is_empty() {
            ExportReceiptQaGateStatus::Passed
        } else {
            ExportReceiptQaGateStatus::Deferred
        };

        Self {
            gate_id: STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_artifact_set_summary(report)),
        }
    }

    #[must_use]
    pub fn stem_package_hash_stability(
        report: &crate::export_qa::StemPackageHashStabilityQaReport,
    ) -> Self {
        let status = if report.status == crate::export_qa::StemPackageHashStabilityQaStatus::Failed
        {
            ExportReceiptQaGateStatus::Failed
        } else if report.deferred_checks.is_empty() {
            ExportReceiptQaGateStatus::Passed
        } else {
            ExportReceiptQaGateStatus::Deferred
        };

        Self {
            gate_id: STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_hash_stability_summary(report)),
        }
    }

    #[must_use]
    pub fn stem_package_non_silence(
        report: &crate::export_qa::StemPackageNonSilenceQaReport,
    ) -> Self {
        let status = match report.status {
            crate::export_qa::StemPackageNonSilenceQaStatus::Passed => {
                ExportReceiptQaGateStatus::Passed
            }
            crate::export_qa::StemPackageNonSilenceQaStatus::Deferred => {
                ExportReceiptQaGateStatus::Deferred
            }
            crate::export_qa::StemPackageNonSilenceQaStatus::Failed => {
                ExportReceiptQaGateStatus::Failed
            }
        };

        Self {
            gate_id: STEM_PACKAGE_NON_SILENCE_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_non_silence_summary(report)),
        }
    }

    #[must_use]
    pub fn stem_package_lineage(report: &crate::export_qa::StemPackageLineageQaReport) -> Self {
        let status = match report.status {
            crate::export_qa::StemPackageLineageQaStatus::Passed => {
                ExportReceiptQaGateStatus::Passed
            }
            crate::export_qa::StemPackageLineageQaStatus::Failed => {
                ExportReceiptQaGateStatus::Failed
            }
        };

        Self {
            gate_id: STEM_PACKAGE_LINEAGE_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_lineage_summary(report)),
        }
    }

    #[must_use]
    pub fn stem_package_fallback_comparison(
        report: &crate::export_qa::StemPackageFallbackComparisonQaReport,
    ) -> Self {
        let status = match report.status {
            crate::export_qa::StemPackageFallbackComparisonQaStatus::Passed => {
                ExportReceiptQaGateStatus::Passed
            }
            crate::export_qa::StemPackageFallbackComparisonQaStatus::Failed => {
                ExportReceiptQaGateStatus::Failed
            }
        };

        Self {
            gate_id: STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_fallback_comparison_summary(report)),
        }
    }

    #[must_use]
    pub fn daw_session_json_package_integrity(
        passed: bool,
        blockers: &[String],
        artifact_roles: Vec<ExportArtifactRole>,
    ) -> Self {
        Self {
            gate_id: DAW_SESSION_JSON_PACKAGE_QA_GATE_ID.into(),
            status: if passed {
                ExportReceiptQaGateStatus::Passed
            } else {
                ExportReceiptQaGateStatus::Failed
            },
            artifact_roles,
            summary: Some(daw_session_json_package_summary(passed, blockers)),
        }
    }

    #[must_use]
    pub fn daw_session_host_import_proof(passed: bool, blockers: &[String]) -> Self {
        Self {
            gate_id: DAW_SESSION_HOST_IMPORT_QA_GATE_ID.into(),
            status: if passed {
                ExportReceiptQaGateStatus::Passed
            } else {
                ExportReceiptQaGateStatus::Failed
            },
            artifact_roles: Vec::new(),
            summary: Some(daw_session_host_import_summary(passed, blockers)),
        }
    }

    #[must_use]
    pub fn daw_session_audible_output_proof(passed: bool, blockers: &[String]) -> Self {
        Self {
            gate_id: DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID.into(),
            status: if passed {
                ExportReceiptQaGateStatus::Passed
            } else {
                ExportReceiptQaGateStatus::Failed
            },
            artifact_roles: Vec::new(),
            summary: Some(daw_session_audible_output_summary(passed, blockers)),
        }
    }
}

fn stem_package_artifact_set_summary(
    report: &crate::export_qa::StemPackageArtifactSetQaReport,
) -> String {
    match report.status {
        crate::export_qa::StemPackageArtifactSetQaStatus::PassedStructureOnly => format!(
            "stem package artifact-set structure accepted for {} claimed stem role(s); {} deferred QA check(s) remain",
            report.claimed_roles.len(),
            report.deferred_checks.len()
        ),
        crate::export_qa::StemPackageArtifactSetQaStatus::Failed => format!(
            "stem package artifact-set evidence failed with {} failure(s); {} deferred QA check(s) remain",
            report.failures.len(),
            report.deferred_checks.len()
        ),
    }
}

fn stem_package_hash_stability_summary(
    report: &crate::export_qa::StemPackageHashStabilityQaReport,
) -> String {
    match report.status {
        crate::export_qa::StemPackageHashStabilityQaStatus::PassedIdentityOnly => format!(
            "stem package per-stem hash identity accepted for {} claimed stem role(s); {} deferred QA check(s) remain",
            report.claimed_roles.len(),
            report.deferred_checks.len()
        ),
        crate::export_qa::StemPackageHashStabilityQaStatus::Failed => format!(
            "stem package per-stem hash stability failed with {} failure(s); {} deferred QA check(s) remain",
            report.failures.len(),
            report.deferred_checks.len()
        ),
    }
}

fn stem_package_non_silence_summary(
    report: &crate::export_qa::StemPackageNonSilenceQaReport,
) -> String {
    match report.status {
        crate::export_qa::StemPackageNonSilenceQaStatus::Passed => format!(
            "stem package per-stem non-silence accepted for {} claimed stem role(s)",
            report.claimed_roles.len()
        ),
        crate::export_qa::StemPackageNonSilenceQaStatus::Deferred => format!(
            "stem package per-stem non-silence deferred with {} missing metric check(s)",
            report.deferred_checks.len()
        ),
        crate::export_qa::StemPackageNonSilenceQaStatus::Failed => format!(
            "stem package per-stem non-silence failed with {} failure(s); {} deferred check(s) remain",
            report.failures.len(),
            report.deferred_checks.len()
        ),
    }
}

fn stem_package_lineage_summary(report: &crate::export_qa::StemPackageLineageQaReport) -> String {
    match report.status {
        crate::export_qa::StemPackageLineageQaStatus::Passed => format!(
            "stem package per-stem lineage accepted for {} claimed stem role(s)",
            report.claimed_roles.len()
        ),
        crate::export_qa::StemPackageLineageQaStatus::Failed => format!(
            "stem package per-stem lineage failed with {} failure(s)",
            report.failures.len()
        ),
    }
}

fn stem_package_fallback_comparison_summary(
    report: &crate::export_qa::StemPackageFallbackComparisonQaReport,
) -> String {
    match report.status {
        crate::export_qa::StemPackageFallbackComparisonQaStatus::Passed => format!(
            "stem package per-stem fallback comparison accepted for {} claimed stem role(s)",
            report.claimed_roles.len()
        ),
        crate::export_qa::StemPackageFallbackComparisonQaStatus::Failed => format!(
            "stem package per-stem fallback comparison failed with {} failure(s)",
            report.failures.len()
        ),
    }
}

fn daw_session_json_package_summary(passed: bool, blockers: &[String]) -> String {
    if passed {
        return "DAW session JSON package integrity accepted for manifest, tempo map, and proof"
            .into();
    }

    format!(
        "DAW session JSON package integrity failed with {} blocker(s): {}",
        blockers.len(),
        blockers.join(", ")
    )
}

fn daw_session_host_import_summary(passed: bool, blockers: &[String]) -> String {
    if passed {
        return "DAW session host import proof accepted".into();
    }

    format!(
        "DAW session host import proof failed with {} blocker(s): {}",
        blockers.len(),
        blockers.join(", ")
    )
}

fn daw_session_audible_output_summary(passed: bool, blockers: &[String]) -> String {
    if passed {
        return "DAW session audible output proof accepted".into();
    }

    format!(
        "DAW session audible output proof failed with {} blocker(s): {}",
        blockers.len(),
        blockers.join(", ")
    )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportReceiptQaGateStatus {
    Passed,
    Failed,
    Deferred,
}

#[cfg(test)]
#[path = "export_qa_gates_tests.rs"]
mod export_qa_gates_tests;
