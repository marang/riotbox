//! Protect the comparator's four referenced inputs before either output write.
//! Naming stays owned by the same helpers used by the manifest writer.

use super::{
    args::Args,
    manifest::{audio_path_for_metrics_path, manifest_path_for_report_path},
    qa_source_safety::reject_source_aliases,
};
use std::{fs, io};

pub(super) fn reject_input_aliases(args: &Args) -> io::Result<()> {
    let outputs = [
        args.report_path.clone(),
        manifest_path_for_report_path(&args.report_path),
    ];
    if outputs[0] == outputs[1] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "report and manifest outputs must be distinct",
        ));
    }
    let inputs = [
        args.baseline_metrics_path.clone(),
        args.candidate_metrics_path.clone(),
        audio_path_for_metrics_path(&args.baseline_metrics_path),
        audio_path_for_metrics_path(&args.candidate_metrics_path),
    ];
    for input in &inputs {
        if !fs::metadata(input)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("input is not a regular file: {}", input.display()),
            ));
        }
        reject_source_aliases(input, outputs.iter().cloned())?;
    }
    // Input preflight already classified both existing destinations. If the
    // report exists, use the same physical identity check between outputs too.
    // Missing report + distinct names cannot alias in a stable namespace.
    match fs::symlink_metadata(&outputs[0]) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(_) => reject_source_aliases(&outputs[0], [outputs[1].clone()]),
    }
}
