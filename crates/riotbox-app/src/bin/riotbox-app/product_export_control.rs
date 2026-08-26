fn execute_product_mix_export(
    shell: &mut JamShellState,
    handoff: Option<&ProductMixExportHandoff>,
    requested_at: u64,
) {
    let Some(handoff) = handoff else {
        let reason = "product mix export unavailable: launch with --product-export-proof and --product-export-destination";
        shell
            .app
            .reject_product_mix_export_request(requested_at, reason);
        shell.set_error_status(reason);
        return;
    };

    match shell.app.commit_product_mix_export_from_active_source_proof(
        &handoff.proof_path,
        &handoff.destination_path,
        requested_at,
    ) {
        Ok(receipt) => shell.set_error_status(format!(
            "exported full_grid_mix | receipt {}",
            receipt.receipt_id
        )),
        Err(error) => {
            shell.set_error_status(format!("product mix export failed: {error}"));
        }
    }
}
