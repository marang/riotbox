use super::args::Args;
use super::artifact_io::{
    metrics_path_for, write_metrics_markdown, write_named_audio_with_metrics,
};
use super::config::{CHANNEL_COUNT, MIN_AFTER_RMS, MIN_DELTA_RMS, MIN_SOURCE_RMS, SAMPLE_RATE};
use super::manifest::{ManifestMetrics, write_manifest};
use super::mix::{before_then_after, feral_after_mix, signal_delta_metrics};
use super::output_paths::PackOutputPaths;
use super::render_plan::{mc202_instigator_state, tr909_fill_state, w30_source_chop_state};
use super::report_markdown::{write_comparison_markdown, write_readme};
use super::source_window::{
    seconds_to_frames, source_preview_from_interleaved, validate_source_format,
};
use riotbox_audio::{
    runtime::{
        render_mc202_offline, render_tr909_offline, render_w30_preview_offline, signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use std::fs;

pub(super) fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let paths = PackOutputPaths::new(&output_dir);
    let stems_dir = output_dir.join("stems");
    fs::create_dir_all(&stems_dir)?;

    let source = SourceAudioCache::load_pcm_wav(&args.source_path)?;
    validate_source_format(&source)?;

    let frame_count = seconds_to_frames(args.duration_seconds);
    let source_window = source.window_by_seconds(args.source_start_seconds, args.duration_seconds);
    if source_window.frame_count < frame_count {
        return Err(format!(
            "source window produced {} frames, but {} are required for {:.3}s",
            source_window.frame_count, frame_count, args.duration_seconds
        )
        .into());
    }

    paths.reject_source_aliases(&args.source_path)?;
    paths.reject_output_aliases()?;
    let source_samples = source.window_samples(source_window).to_vec();
    write_interleaved_pcm16_wav(
        &paths.source_excerpt,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &source_samples,
    )?;

    let w30_source_window = source.window_by_seconds(
        args.source_start_seconds,
        args.source_window_seconds.min(args.duration_seconds),
    );
    let w30_preview = source_preview_from_interleaved(
        source.window_samples(w30_source_window),
        usize::from(CHANNEL_COUNT),
        w30_source_window.start_frame as u64,
        w30_source_window
            .start_frame
            .saturating_add(w30_source_window.frame_count) as u64,
    )
    .ok_or("source-backed W-30 preview window produced no samples")?;

    let w30 = render_w30_preview_offline(
        &w30_source_chop_state(w30_preview),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let tr909 = render_tr909_offline(&tr909_fill_state(), SAMPLE_RATE, CHANNEL_COUNT, frame_count);
    let mc202 = render_mc202_offline(
        &mc202_instigator_state(),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let after = feral_after_mix(&source_samples, &w30, &tr909, &mc202);
    let before_then_after = before_then_after(&source_samples, &after);

    write_named_audio_with_metrics(&paths.w30, &w30)?;
    write_named_audio_with_metrics(&paths.tr909, &tr909)?;
    write_named_audio_with_metrics(&paths.mc202, &mc202)?;
    write_named_audio_with_metrics(&paths.after, &after)?;
    write_interleaved_pcm16_wav(
        &paths.before_after,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &before_then_after,
    )?;

    let source_metrics = signal_metrics(&source_samples);
    let after_metrics = signal_metrics(&after);
    let delta_metrics = signal_delta_metrics(&source_samples, &after);
    let w30_metrics = signal_metrics(&w30);
    let tr909_metrics = signal_metrics(&tr909);
    let mc202_metrics = signal_metrics(&mc202);
    write_metrics_markdown(&metrics_path_for(&paths.source_excerpt), source_metrics)?;
    write_comparison_markdown(
        &paths.comparison,
        source_metrics,
        after_metrics,
        delta_metrics,
    )?;
    write_readme(&paths.readme, args, &paths.source_excerpt)?;

    if source_metrics.rms <= MIN_SOURCE_RMS {
        return Err("source excerpt rendered near silence".into());
    }
    if after_metrics.rms <= MIN_AFTER_RMS {
        return Err("Riotbox Feral after render produced near silence".into());
    }
    if delta_metrics.rms <= MIN_DELTA_RMS {
        return Err(format!(
            "Riotbox Feral after render is too similar to source: delta RMS {:.6}",
            delta_metrics.rms
        )
        .into());
    }

    write_manifest(
        args,
        &paths,
        ManifestMetrics {
            source_excerpt: source_metrics.into(),
            riotbox_after: after_metrics.into(),
            source_after_delta: delta_metrics.into(),
            w30_source_chop: w30_metrics.into(),
            tr909_fill: tr909_metrics.into(),
            mc202_instigator: mc202_metrics.into(),
        },
    )?;

    Ok(())
}
