mod arrangement_export_placement;
mod daw_tempo_map;
mod export_qa_gates;
mod export_types;

pub use arrangement_export_placement::{
    ArrangementExportPlacementReadinessBlocker, ArrangementExportPlacementReadinessReport,
    ArrangementExportPlacementReadinessStatus, ExportArrangementPlacementRef,
    validate_arrangement_export_placement_readiness,
};
pub use daw_tempo_map::{
    DawTempoMapReadinessBlocker, DawTempoMapReadinessReport, DawTempoMapReadinessStatus,
    ExportDawTempoMapRef, validate_daw_tempo_map_readiness,
};
pub use export_qa_gates::{
    DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID, DAW_SESSION_HOST_IMPORT_QA_GATE_ID,
    DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, ExportReceiptQaGateResult, ExportReceiptQaGateStatus,
    PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
    STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID, STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
    STEM_PACKAGE_LINEAGE_QA_GATE_ID, STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
};
pub use export_types::{
    ExportArtifactAudioMetrics, ExportArtifactFallbackComparisonEvidence,
    ExportArtifactFallbackComparisonKind, ExportArtifactLocation, ExportArtifactMediaType,
    ExportArtifactRole, ExportArtifactSetEntry, ExportArtifactSourceGraphRef,
    ExportArtifactTimingGridRef, ExportReceiptState, StemPackageReceiptReadinessBlocker,
    StemPackageReceiptReadinessReport, StemPackageReceiptReadinessStatus,
    validate_stem_package_receipt_readiness,
};

// Textual includes keep this large file split mechanical and behavior-preserving.
include!("session/version_types.rs");
include!("session/mc202_types.rs");
include!("session/defaults.rs");
