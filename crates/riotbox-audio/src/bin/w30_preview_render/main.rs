mod args;
mod artifact_io;
mod config;
#[path = "../qa_source_safety/mod.rs"]
mod qa_source_safety;
mod source_window;
#[cfg(test)]
mod tests;

use args::{Args, print_help};
use artifact_io::{metrics_path_for, write_metrics_markdown, write_pcm16_wav};
use config::{CHANNEL_COUNT, SAMPLE_RATE};
use riotbox_audio::runtime::{render_w30_preview_offline, signal_metrics};
use source_window::source_window_smoke_state;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let frame_count = args.render_frame_count()?;
    let source_window_preview = args.source_window_preview()?;
    let metrics_path = metrics_path_for(&args.output_path);
    if let Some(source) = &args.source_path {
        qa_source_safety::reject_source_aliases(
            source,
            [args.output_path.clone(), metrics_path.clone()],
        )?;
    }
    let samples = render_w30_preview_offline(
        &source_window_smoke_state(source_window_preview),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
    );
    let metrics = signal_metrics(&samples);

    write_pcm16_wav(&args.output_path, SAMPLE_RATE, CHANNEL_COUNT, &samples)?;

    write_metrics_markdown(&metrics_path, &args, samples.len(), metrics)?;

    println!("wrote {}", args.output_path.display());
    println!("wrote {}", metrics_path.display());

    Ok(())
}
