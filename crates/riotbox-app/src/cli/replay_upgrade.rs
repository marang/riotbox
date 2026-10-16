//! Presentation only: Core owns chronology admission, fidelity and publication.
use std::{
    error::Error,
    io::{self, Write},
};

use riotbox_core::persistence::{ReplayChronologyCopy, copy_session_with_committed_order};
use serde::Serialize;

use super::model::{AppLaunch, LaunchMode};

#[derive(Serialize)]
struct Summary<'a> {
    mode: &'static str,
    #[serde(flatten)]
    copy: &'a ReplayChronologyCopy,
    input_modified: bool,
    references_verified: bool,
    older_reader_warning: &'static str,
}

pub(super) fn run(
    launch: &AppLaunch,
    output: &mut impl Write,
) -> Result<ReplayChronologyCopy, Box<dyn Error>> {
    let LaunchMode::ReplayChronologyUpgradeCopy {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a replay chronology upgrade copy launch".into());
    };
    let copy = copy_session_with_committed_order(session_path, destination_path)?;
    let summary = Summary {
        mode: "replay_chronology_upgrade_copy",
        copy: &copy,
        input_modified: false,
        references_verified: false,
        older_reader_warning: "Older Riotbox versions reject the committed-order replay policy; the original Session is unchanged.",
    };
    let write_result = (|| -> Result<(), Box<dyn Error>> {
        serde_json::to_writer_pretty(&mut *output, &summary)?;
        writeln!(output)?;
        output.flush()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        return Err(io::Error::other(format!(
            "destination {} was published, but writing the result to stdout failed: {error}",
            copy.destination_path.display()
        ))
        .into());
    }
    Ok(copy)
}
