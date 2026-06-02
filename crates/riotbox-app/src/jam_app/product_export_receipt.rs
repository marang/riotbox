use riotbox_core::session::{
    ExportArtifactRole, ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef,
    ExportReceiptState, SessionFile,
};

pub(super) fn attach_product_export_artifact_lineage(
    receipt: &mut ExportReceiptState,
    session: &SessionFile,
) {
    if let Some(source_graph_ref) = export_artifact_source_graph_ref(session) {
        receipt.attach_artifact_source_graph_ref(ExportArtifactRole::FullGridMix, source_graph_ref);
    }
    if let Some(timing_grid_ref) = export_artifact_timing_grid_ref(session) {
        receipt.attach_artifact_timing_grid_ref(ExportArtifactRole::FullGridMix, timing_grid_ref);
    }
}

fn export_artifact_source_graph_ref(session: &SessionFile) -> Option<ExportArtifactSourceGraphRef> {
    session
        .source_graph_refs
        .first()
        .map(|graph_ref| ExportArtifactSourceGraphRef {
            source_id: graph_ref.source_id.clone(),
            graph_version: graph_ref.graph_version,
            graph_hash: graph_ref.graph_hash.clone(),
        })
}

fn export_artifact_timing_grid_ref(session: &SessionFile) -> Option<ExportArtifactTimingGridRef> {
    session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .map(|confirmed_grid| ExportArtifactTimingGridRef {
            source_id: confirmed_grid.source_id.clone(),
            hypothesis_id: confirmed_grid.hypothesis_id.clone(),
            confirmed_by_action: confirmed_grid.confirmed_by_action,
            confirmed_at: confirmed_grid.confirmed_at,
        })
}
