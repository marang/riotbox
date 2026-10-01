mod args;
mod artifact_io;
mod config;
mod manifest;
mod mix;
mod output_paths;
mod pack_builder;
#[path = "../qa_source_safety/mod.rs"]
mod qa_source_safety;
mod render_plan;
mod report_markdown;
mod source_window;
#[cfg(test)]
mod tests;

use args::{Args, print_help};
use pack_builder::render_pack;
use std::env;

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
