use args::Args;
use args::print_help;
use pack_builder::render_pack;
use std::env;

mod args;
mod artifact_io;
mod case_catalog;
mod config;
mod manifest;
mod mc202_phrase_grid;
mod mc202_source_phrase_slot;
mod pack_builder;
mod render_case;
mod report_markdown;
mod report_model;
mod signal_delta;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    render_pack(&args)?;
    println!("wrote {}", args.output_dir().display());

    Ok(())
}

#[cfg(test)]
mod mc202_phrase_grid_consumer_tests;
#[cfg(test)]
mod tests;
