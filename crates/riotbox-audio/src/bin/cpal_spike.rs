use std::time::Duration;

use riotbox_audio::probe::run_output_probe;

fn main() {
    let summary = run_output_probe(Duration::from_millis(250));
    println!("{summary}");
}
