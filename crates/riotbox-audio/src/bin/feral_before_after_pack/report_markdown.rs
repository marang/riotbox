use super::args::Args;
use super::config::{MIN_AFTER_RMS, MIN_DELTA_RMS, MIN_SOURCE_RMS, PACK_ID};
use riotbox_audio::runtime::OfflineAudioMetrics;
use std::{fs, path::Path};

pub(super) fn write_comparison_markdown(
    path: &Path,
    source: OfflineAudioMetrics,
    after: OfflineAudioMetrics,
    delta: OfflineAudioMetrics,
) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Comparison\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source RMS: `{:.6}`\n\
             - Riotbox after RMS: `{:.6}`\n\
             - Signal delta RMS: `{:.6}`\n\
             - Source peak abs: `{:.6}`\n\
             - Riotbox after peak abs: `{:.6}`\n\
             - Signal delta peak abs: `{:.6}`\n\
             - Result: `{}`\n",
            source.rms,
            after.rms,
            delta.rms,
            source.peak_abs,
            after.peak_abs,
            delta.peak_abs,
            if source.rms > MIN_SOURCE_RMS && after.rms > MIN_AFTER_RMS && delta.rms > MIN_DELTA_RMS
            {
                "pass"
            } else {
                "fail"
            }
        ),
    )
}

pub(super) fn write_readme(
    path: &Path,
    args: &Args,
    source_excerpt_path: &Path,
) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Pack\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source: `{}`\n\
             - Source window: `{:.3}s` to `{:.3}s`\n\
             - Source preview window for W-30: `{:.3}s`\n\n\
             ## Files\n\n\
             - `01_source_excerpt.wav`: direct source excerpt.\n\
             - `02_riotbox_feral_changed.wav`: Riotbox-rendered Feral preview mix.\n\
             - `03_before_then_after.wav`: source excerpt, short silence, then Riotbox after render.\n\
             - `comparison.md`: source-vs-after metrics.\n\n\
             - `manifest.json`: machine-readable artifact paths, thresholds, and metrics.\n\n\
             ## Stems\n\n\
             - `stems/w30_source_chop.wav`: source-backed W-30 preview render.\n\
             - `stems/tr909_fill.wav`: TR-909 fill render.\n\
             - `stems/mc202_instigator.wav`: MC-202 instigator render.\n\n\
             ## Current Limit\n\n\
             This pack proves a current offline listening/QA path. It does not claim the live TUI mixer can perform this combined result directly yet.\n\n\
             ## Source Excerpt\n\n\
             `{}`\n",
            args.source_path.display(),
            args.source_start_seconds,
            args.source_start_seconds + args.duration_seconds,
            args.source_window_seconds.min(args.duration_seconds),
            source_excerpt_path.display()
        ),
    )
}
