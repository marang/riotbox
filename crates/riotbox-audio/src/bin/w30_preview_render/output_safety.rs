//! The WAV and its actual metrics destination must stay distinct on disk.
//! Applies to explicit source and explicit synthetic diagnostic modes alike.

use super::qa_source_safety::{existing_output_is_regular, reject_source_aliases};
use std::{io, path::Path};

pub(super) fn reject_output_aliases(wav: &Path, metrics: &Path) -> io::Result<()> {
    let wav_exists = existing_output_is_regular(wav)?;
    let metrics_exists = existing_output_is_regular(metrics)?;
    // Their writer-derived names are distinct. In a stable namespace, physical
    // aliasing is possible only when both entries already exist.
    if wav_exists && metrics_exists {
        reject_source_aliases(wav, [metrics.to_path_buf()])?;
    }
    Ok(())
}
