//! Literal POSIX reproduction command; never executes a shell or renders audio.
use super::{args::Args, grid::Grid};

pub(super) fn verification_command(args: &Args, grid: &Grid, source_window_seconds: f32) -> String {
    let bpm_arg = if args.bpm_overridden {
        format!(" {:.3}", grid.bpm)
    } else {
        " auto".to_string()
    };
    format!(
        "just feral-grid-pack {} {}{} {} {:.3} {:.3}",
        quote_posix_argument(&args.source_path.display().to_string()),
        quote_posix_argument(&args.date),
        bpm_arg,
        grid.bars,
        source_window_seconds,
        args.source_start_seconds
    )
}

fn quote_posix_argument(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
