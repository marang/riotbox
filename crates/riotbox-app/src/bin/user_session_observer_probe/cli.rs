use std::path::PathBuf;

const SUPPORTED_PROBES: &str = "recipe2-mc202|first-playable-jam|stage-style-jam|stage-style-restore-diversity|interrupted-session-recovery|missing-target-recovery|feral-grid-jam|feral-grid-jam-fallback|feral-grid-jam-locked|source-timing-confirmation|source-transport-map-capture";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Args {
    pub(super) probe: String,
    pub(super) observer_path: PathBuf,
    pub(super) show_help: bool,
}

impl Args {
    pub(super) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut probe = None;
        let mut observer_path = None;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--probe" => {
                    probe = Some(
                        args.next()
                            .ok_or_else(|| "--probe requires a value".to_string())?,
                    );
                }
                "--observer" => {
                    observer_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--observer requires a path".to_string())?,
                    ));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if show_help {
            return Ok(Self {
                probe: String::new(),
                observer_path: PathBuf::new(),
                show_help,
            });
        }

        Ok(Self {
            probe: probe.ok_or_else(|| "--probe is required".to_string())?,
            observer_path: observer_path.ok_or_else(|| "--observer is required".to_string())?,
            show_help,
        })
    }
}

pub(super) fn print_help() {
    println!(
        "Usage:\n  user_session_observer_probe --probe <{SUPPORTED_PROBES}> --observer <events.ndjson>"
    );
}
