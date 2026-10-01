//! Exact output layout and source-preservation preflight for this offline binary.
//! This is not a concurrent namespace lock or an atomic pack transaction.

use std::{
    io,
    path::{Path, PathBuf},
};

use super::qa_source_safety::reject_source_aliases;

pub(super) struct PackOutputPaths {
    pub(super) tr909: PathBuf,
    pub(super) w30: PathBuf,
    pub(super) mc202: PathBuf,
    pub(super) product_drums: PathBuf,
    pub(super) product_music: PathBuf,
    pub(super) product_bass: PathBuf,
    pub(super) source_first_mix: PathBuf,
    pub(super) full_mix: PathBuf,
    pub(super) report: PathBuf,
    pub(super) manifest: PathBuf,
    pub(super) readme: PathBuf,
}

impl PackOutputPaths {
    pub(super) fn new(output: &Path) -> Self {
        let stems = output.join("stems");
        let product = stems.join("product");
        Self {
            tr909: stems.join("01_tr909_beat_fill.wav"),
            w30: stems.join("02_w30_feral_source_chop.wav"),
            mc202: stems.join("03_mc202_bass_pressure.wav"),
            product_drums: product.join("01_stem_drums.wav"),
            product_music: product.join("02_stem_music.wav"),
            product_bass: product.join("03_stem_bass.wav"),
            source_first_mix: output.join("04_riotbox_source_first_mix.wav"),
            full_mix: output.join("05_riotbox_generated_support_mix.wav"),
            report: output.join("grid-report.md"),
            manifest: output.join("manifest.json"),
            readme: output.join("README.md"),
        }
    }

    pub(super) fn reject_source_aliases(&self, source: &Path) -> io::Result<()> {
        let audio = [
            &self.tr909,
            &self.w30,
            &self.mc202,
            &self.product_drums,
            &self.product_music,
            &self.product_bass,
            &self.source_first_mix,
            &self.full_mix,
        ];
        let artifacts = audio
            .into_iter()
            .flat_map(|path| [path.clone(), metrics_path_for(path)])
            .chain([
                self.report.clone(),
                self.manifest.clone(),
                self.readme.clone(),
            ]);
        reject_source_aliases(source, artifacts)
    }
}

pub(super) fn metrics_path_for(path: &Path) -> PathBuf {
    let mut metrics_path = path.to_path_buf();
    metrics_path.set_file_name(match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) => format!("{stem}.metrics.md"),
        None => "metrics.md".to_string(),
    });
    metrics_path
}
