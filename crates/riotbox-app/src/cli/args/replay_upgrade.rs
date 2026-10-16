//! Exclusive metadata-copy arguments, separate from interactive launch options.
use std::path::PathBuf;

use crate::cli::model::{AppLaunch, LaunchMode};

const MODE: &str = "--replay-chronology-upgrade-copy";
const DESTINATION: &str = "--replay-chronology-destination";

pub(super) fn parse(arguments: &[String]) -> Option<Result<AppLaunch, String>> {
    arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), MODE | DESTINATION))
        .then(|| parse_copy(arguments))
}

fn parse_copy(arguments: &[String]) -> Result<AppLaunch, String> {
    let mut args = arguments.iter().map(String::as_str);
    let mut enabled = false;
    let mut session_path = None;
    let mut destination_path = None;
    while let Some(argument) = args.next() {
        match argument {
            MODE if !enabled => enabled = true,
            MODE => return Err(format!("{MODE} must appear exactly once")),
            "--session" if session_path.is_none() => {
                session_path = Some(path(&mut args, "--session")?);
            }
            "--session" => return Err("--session must appear exactly once".into()),
            DESTINATION if destination_path.is_none() => {
                destination_path = Some(path(&mut args, DESTINATION)?);
            }
            DESTINATION => return Err(format!("{DESTINATION} must appear exactly once")),
            other => {
                return Err(format!(
                    "{MODE} is exclusive; unsupported argument: {other}"
                ));
            }
        }
    }
    if !enabled {
        return Err(format!("{DESTINATION} requires {MODE}"));
    }
    let session_path =
        session_path.ok_or_else(|| format!("{MODE} requires --session <input.json>"))?;
    let destination_path = destination_path
        .ok_or_else(|| format!("{MODE} requires {DESTINATION} <new-sibling.json>"))?;
    Ok(AppLaunch {
        mode: LaunchMode::ReplayChronologyUpgradeCopy {
            session_path,
            destination_path,
        },
        observer_path: None,
    })
}

fn path<'a>(args: &mut impl Iterator<Item = &'a str>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .filter(|value| !value.is_empty() && !value.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing path for {flag}; prefix a leading-dash filename with ./"))
}
