//! The actual fourteen-file layout, shared by preflight, writers and manifest.

use super::artifact_io::metrics_path_for;
use super::qa_source_safety::{reject_output_aliases, reject_source_aliases};
use std::{
    io,
    path::{Path, PathBuf},
};

pub(super) struct PackOutputPaths {
    pub(super) source_excerpt: PathBuf,
    pub(super) after: PathBuf,
    pub(super) before_after: PathBuf,
    pub(super) w30: PathBuf,
    pub(super) tr909: PathBuf,
    pub(super) mc202: PathBuf,
    pub(super) comparison: PathBuf,
    pub(super) readme: PathBuf,
    pub(super) manifest: PathBuf,
}

impl PackOutputPaths {
    pub(super) fn new(output: &Path) -> Self {
        let stems = output.join("stems");
        Self {
            source_excerpt: output.join("01_source_excerpt.wav"),
            after: output.join("02_riotbox_feral_changed.wav"),
            before_after: output.join("03_before_then_after.wav"),
            w30: stems.join("w30_source_chop.wav"),
            tr909: stems.join("tr909_fill.wav"),
            mc202: stems.join("mc202_instigator.wav"),
            comparison: output.join("comparison.md"),
            readme: output.join("README.md"),
            manifest: output.join("manifest.json"),
        }
    }

    pub(super) fn reject_source_aliases(&self, source: &Path) -> io::Result<()> {
        reject_source_aliases(source, self.artifacts())
    }

    pub(super) fn reject_output_aliases(&self) -> io::Result<()> {
        reject_output_aliases(self.artifacts())
    }

    fn artifacts(&self) -> impl Iterator<Item = PathBuf> + '_ {
        let audio_with_metrics = [
            &self.source_excerpt,
            &self.after,
            &self.w30,
            &self.tr909,
            &self.mc202,
        ];
        audio_with_metrics
            .into_iter()
            .flat_map(|path| [path.clone(), metrics_path_for(path)])
            .chain([
                self.before_after.clone(),
                self.comparison.clone(),
                self.readme.clone(),
                self.manifest.clone(),
            ])
    }
}
