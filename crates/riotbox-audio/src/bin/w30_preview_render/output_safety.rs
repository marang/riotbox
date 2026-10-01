//! The WAV and its actual metrics destination must stay distinct on disk.
//! Applies to explicit source and explicit synthetic diagnostic modes alike.

use super::qa_source_safety;
use std::{io, path::Path};

pub(super) fn reject_output_aliases(wav: &Path, metrics: &Path) -> io::Result<()> {
    qa_source_safety::reject_output_aliases([wav.to_path_buf(), metrics.to_path_buf()])
}
