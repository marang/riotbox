use std::{error::Error, fs::File, path::PathBuf};

use riotbox_app::ui::perform_risk_cue_contract::perform_risk_cue_contract;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let output = match args.next() {
        Some(flag) if flag == "--output" => args.next().map(PathBuf::from),
        Some(flag) => {
            return Err(format!("unknown argument: {}", flag.to_string_lossy()).into());
        }
        None => None,
    };
    if args.next().is_some() {
        return Err("unexpected trailing arguments".into());
    }

    let contract = perform_risk_cue_contract();
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &contract)?;
    } else {
        serde_json::to_writer_pretty(std::io::stdout(), &contract)?;
    }
    Ok(())
}
