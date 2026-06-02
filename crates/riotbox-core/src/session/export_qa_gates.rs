use serde::{Deserialize, Serialize};

use super::export_types::ExportArtifactRole;

pub const PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID: &str = "product_export_reproducibility_smoke";
pub const STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID: &str = "stem_package_artifact_set_evidence";
pub const STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID: &str = "stem_package_per_stem_hash_stability";

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportReceiptQaGateStatus {
    Passed,
    Failed,
    Deferred,
}
